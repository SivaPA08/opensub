use base64::Engine;
use headless_chrome::{Browser, LaunchOptionsBuilder, protocol::cdp::Page};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct PyResponse {
    pub status: String,
    pub message: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RenderConfig {
    pub input_path: String,
    pub output_path: String,
    pub subtitles: Vec<RenderSubtitle>,
    pub style: RenderStyle,
    pub position: Option<RenderPosition>,
    pub video_info: Option<VideoInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderSubtitle {
    pub content: String,
    pub start: f32,
    pub end: f32,
    pub font_size: Option<f32>,
    pub font_color: Option<String>,
    pub background_color: Option<String>,
    pub custom_font: Option<String>,
    pub font_opacity: Option<f32>,
    pub background_opacity: Option<f32>,
    pub animation_type: Option<String>,
    pub animation_speed: Option<f32>,
    pub sub_x: Option<f32>,
    pub sub_y: Option<f32>,
    pub sub_width: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderStyle {
    pub font_size: Option<f32>,
    pub font_color: Option<String>,
    pub background_color: Option<String>,
    pub custom_font: Option<String>,
    pub font_opacity: Option<f32>,
    pub background_opacity: Option<f32>,
    pub animation_type: Option<String>,
    pub animation_speed: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPosition {
    pub sub_x: Option<f32>,
    pub sub_y: Option<f32>,
    pub sub_width: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VideoInfo {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub container_width: Option<f32>,
    pub container_height: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RenderStateStyle {
    fontSize: f32,
    fontColor: String,
    backgroundColor: String,
    customFont: String,
    fontOpacity: f32,
    backgroundOpacity: f32,
    animationType: String,
    animationSpeed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RenderStateSub {
    content: String,
    subX: f32,
    subY: f32,
    subWidth: f32,
    timeOffset: f32,
    style: RenderStateStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RenderState {
    subtitles: Vec<RenderStateSub>,
    scaleFactor: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    customFontBase64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    customFontName: Option<String>,
}

pub fn get_ffmpeg_command(app: &AppHandle) -> Result<Command, String> {
    #[cfg(debug_assertions)]
    {
        let _ = app;
        Ok(Command::new("ffmpeg"))
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = app;
        let exe_dir = std::env::current_exe()
            .map_err(|e| format!("Failed to get current exe path: {e}"))?
            .parent()
            .ok_or("Failed to get exe parent directory")?
            .to_path_buf();
        let target_triple = env!("TARGET_TRIPLE");
        let ext = if cfg!(target_os = "windows") { ".exe" } else { "" };
        let sidecar_name = format!("ffmpeg-{}{}", target_triple, ext);
        let path = exe_dir.join(&sidecar_name);
        Ok(Command::new(path))
    }
}

pub fn get_ffprobe_command(app: &AppHandle) -> Result<Command, String> {
    #[cfg(debug_assertions)]
    {
        let _ = app;
        Ok(Command::new("ffprobe"))
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = app;
        let exe_dir = std::env::current_exe()
            .map_err(|e| format!("Failed to get current exe path: {e}"))?
            .parent()
            .ok_or("Failed to get exe parent directory")?
            .to_path_buf();
        let target_triple = env!("TARGET_TRIPLE");
        let ext = if cfg!(target_os = "windows") { ".exe" } else { "" };
        let sidecar_name = format!("ffprobe-{}{}", target_triple, ext);
        let path = exe_dir.join(&sidecar_name);
        Ok(Command::new(path))
    }
}

fn ffprobe_info(app: &AppHandle, path: &str) -> Result<(u32, u32, f32, f32), String> {
    let out = get_ffprobe_command(app)?
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_streams",
            "-show_format",
            path,
        ])
        .output()
        .map_err(|e| format!("ffprobe failed: {e}"))?;

    if !out.status.success() {
        return Err("ffprobe returned a non-zero exit code".to_string());
    }

    let v: serde_json::Value =
        serde_json::from_slice(&out.stdout).map_err(|e| format!("bad ffprobe json: {e}"))?;

    let mut width = 1280;
    let mut height = 720;
    let mut fps = 30.0f32;
    let mut duration = 0.0f32;

    if let Some(d) = v["format"]["duration"].as_str() {
        duration = d.parse::<f32>().unwrap_or(0.0);
    }

    for stream in v["streams"].as_array().unwrap_or(&vec![]) {
        if stream["codec_type"].as_str() == Some("video") {
            if let Some(w) = stream["width"].as_u64() {
                width = w as u32;
            }
            if let Some(h) = stream["height"].as_u64() {
                height = h as u32;
            }

            if let Some(fps_str) = stream["avg_frame_rate"].as_str() {
                if let Some((n, d)) = fps_str.split_once('/') {
                    let n = n.parse::<f32>().unwrap_or(30.0);
                    let d = d.parse::<f32>().unwrap_or(1.0);
                    if d > 0.0 {
                        fps = n / d;
                    }
                } else {
                    fps = fps_str.parse::<f32>().unwrap_or(30.0);
                }
            }

            if let Some(d) = stream["duration"].as_str() {
                duration = d.parse::<f32>().unwrap_or(duration);
            }
            break;
        }
    }

    Ok((width, height, fps, duration))
}

fn mime_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

fn serve_one_connection(mut stream: TcpStream, build_dir: &Path) -> Result<(), String> {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).map_err(|e| e.to_string())?;
    if n == 0 {
        return Ok(());
    }

    let req = String::from_utf8_lossy(&buf[..n]);
    let path = req
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    let path_clean = path.split('?').next().unwrap_or(path).split('#').next().unwrap_or(path);
    let rel = path_clean.trim_start_matches('/');
    let candidate = if rel.is_empty() {
        build_dir.join("index.html")
    } else {
        build_dir.join(rel)
    };

    let file_path = if candidate.is_file() {
        candidate
    } else {
        build_dir.join("index.html")
    };

    let body = fs::read(&file_path).map_err(|e| format!("failed to read {:?}: {}", file_path, e))?;
    let mime = mime_for_path(&file_path);

    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        body.len(),
        mime
    );

    stream
        .write_all(header.as_bytes())
        .map_err(|e| e.to_string())?;
    stream.write_all(&body).map_err(|e| e.to_string())?;
    Ok(())
}

fn start_static_server(build_dir: PathBuf) -> Result<(u16, thread::JoinHandle<()>), String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();

    let handle = thread::spawn(move || {
        for incoming in listener.incoming() {
            if let Ok(stream) = incoming {
                let dir = build_dir.clone();
                thread::spawn(move || {
                    let _ = serve_one_connection(stream, &dir);
                });
            }
        }
    });

    Ok((port, handle))
}

fn resolve_build_dir(app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    {
        let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or("bad CARGO_MANIFEST_DIR")?
            .join("build");

        if dev.join("index.html").exists() {
            return Ok(dev);
        }
    }

    let res = app.path().resource_dir().map_err(|e| e.to_string())?;
    let bundled = res.join("build");
    if bundled.join("index.html").exists() {
        return Ok(bundled);
    }

    Err(format!(
        "Svelte build directory not found (index.html missing): {:?}",
        bundled
    ))
}

fn render_video(app: &AppHandle, config: RenderConfig) -> Result<PyResponse, String> {
    let (probe_width, probe_height, fps, duration) = ffprobe_info(app, &config.input_path)?;

    let width = config
        .video_info
        .as_ref()
        .and_then(|v| v.width)
        .map(|w| w.round() as u32)
        .filter(|&w| w > 0)
        .unwrap_or(probe_width);
    let height = config
        .video_info
        .as_ref()
        .and_then(|v| v.height)
        .map(|h| h.round() as u32)
        .filter(|&h| h > 0)
        .unwrap_or(probe_height);

    let scale_factor = config
        .video_info
        .as_ref()
        .and_then(|v| {
            let video_h = v.height.filter(|&h| h > 0.0).unwrap_or(height as f32);
            if let Some(container_h) = v.container_height.filter(|&h| h > 0.0) {
                return Some(video_h / container_h);
            }
            if let (Some(container_w), Some(video_w)) = (
                v.container_width.filter(|&w| w > 0.0),
                v.width.filter(|&w| w > 0.0),
            ) {
                let container_h = container_w * video_h / video_w;
                return Some(video_h / container_h);
            }
            None
        })
        .unwrap_or(1.0);

    let mut render_url = None;
    let mut _server_handle = None;

    #[cfg(debug_assertions)]
    {
        // Try to connect to Vite dev server (trying localhost, IPv4, and IPv6 loopbacks)
        let targets = ["localhost:1420", "127.0.0.1:1420", "[::1]:1420"];
        for target in &targets {
            if let Ok(std_stream) = std::net::TcpStream::connect(target) {
                render_url = Some(format!("http://{}/render", target));
                drop(std_stream);
                break;
            }
        }
    }

    let render_url = if let Some(url) = render_url {
        url
    } else {
        let build_dir = resolve_build_dir(app)?;
        let (port, handle) = start_static_server(build_dir)?;
        _server_handle = Some(handle);
        format!("http://127.0.0.1:{}/render", port)
    };

    let launch_options = LaunchOptionsBuilder::default()
        .headless(true)
        .window_size(Some((width, height)))
        .build()
        .map_err(|e| format!("failed to build Chrome options: {}", e))?;

    let browser = Browser::new(launch_options).map_err(|e| format!("failed to launch browser: {}", e))?;
    let tab = browser
        .new_tab()
        .map_err(|e| format!("failed to open tab: {}", e))?;

    tab.set_transparent_background_color()
        .map_err(|e| format!("failed to set transparent background color: {}", e))?;

    tab.call_method(headless_chrome::protocol::cdp::Emulation::SetDeviceMetricsOverride {
        width,
        height,
        device_scale_factor: 1.0,
        mobile: false,
        scale: None,
        screen_width: None,
        screen_height: None,
        position_x: None,
        position_y: None,
        dont_set_visible_size: None,
        screen_orientation: None,
        viewport: None,
        display_feature: None,
        device_posture: None,
    }).map_err(|e| format!("failed to set device metrics: {}", e))?;

    tab.navigate_to(&render_url)
        .map_err(|e| format!("failed to navigate to render page: {}", e))?;
    tab.wait_until_navigated()
        .map_err(|e| format!("failed waiting for navigation: {}", e))?;

    for _ in 0..200 {
        let ready = tab
            .evaluate("window.renderPageReady === true", false)
            .map_err(|e| format!("failed to check readiness: {}", e))?;

        if ready
            .value
            .as_ref()
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            break;
        }

        thread::sleep(Duration::from_millis(50));
    }

    let mut ffmpeg = get_ffmpeg_command(app)?
        .args([
            "-y",
            "-i",
            &config.input_path,
            "-f",
            "image2pipe",
            "-framerate",
            &format!("{fps:.6}"),
            "-vcodec",
            "png",
            "-i",
            "-",
            "-filter_complex",
            "[0:v][1:v]overlay=0:0",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-map",
            "0:a?",
            "-progress",
            "pipe:1",
            &config.output_path,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn ffmpeg: {e}"))?;

    let mut ffmpeg_stdin = ffmpeg
        .stdin
        .take()
        .ok_or("failed to capture ffmpeg stdin")?;

    let stdout = ffmpeg
        .stdout
        .take()
        .ok_or("failed to capture ffmpeg stdout")?;
    let stderr = ffmpeg
        .stderr
        .take()
        .ok_or("failed to capture ffmpeg stderr")?;

    let app_clone = app.clone();
    let stdout_handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for _ in reader.lines().flatten() {}
    });

    let stderr_log = Arc::new(Mutex::new(String::new()));
    let stderr_log_clone = stderr_log.clone();
    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            let mut log = stderr_log_clone.lock().unwrap();
            log.push_str(&line);
            log.push('\n');
        }
    });

    let pos = config.position.clone().unwrap_or(RenderPosition {
        sub_x: Some(50.0),
        sub_y: Some(85.0),
        sub_width: Some(70.0),
    });

    let total_frames = (duration * fps).round().max(1.0) as u32;
    let frame_interval_ms = (1000.0 / fps).round() as u64;

    let mut prev_sub_id: Option<Vec<(f32, f32, String)>> = None;
    let mut empty_frame_bytes: Option<Vec<u8>> = None;
    let mut active_frame_bytes: Option<Vec<u8>> = None;

    for frame_idx in 0..total_frames {
        let t = frame_idx as f32 / fps;

        let active_subs: Vec<&RenderSubtitle> = config
            .subtitles
            .iter()
            .filter(|sub| t >= sub.start && t <= sub.end)
            .collect();

        let sub_ids: Vec<(f32, f32, String)> = active_subs
            .iter()
            .map(|sub| (sub.start, sub.end, sub.content.clone()))
            .collect();

        let mut is_animated = false;
        for sub in &active_subs {
            let anim_type = sub.animation_type.as_deref()
                .or(config.style.animation_type.as_deref())
                .unwrap_or("none");
            if !matches!(anim_type, "none" | "" | "scale-in") {
                is_animated = true;
                break;
            }
        }

        let frame_bytes = if active_subs.is_empty() {
            if prev_sub_id.is_some() || empty_frame_bytes.is_none() {
                let state = RenderState {
                    subtitles: vec![],
                    scaleFactor: scale_factor,
                    customFontBase64: None,
                    customFontName: None,
                };

                let js = format!("window.setRenderState({});", serde_json::to_string(&state).map_err(|e| e.to_string())?);
                tab.evaluate(&js, false)
                    .map_err(|e| format!("failed to clear render state: {}", e))?;

                empty_frame_bytes = Some(
                    tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)
                        .map_err(|e| format!("failed to capture empty frame: {}", e))?,
                );
                prev_sub_id = None;
            }

            empty_frame_bytes.clone().unwrap()
        } else {
            let mut active_payload = Vec::with_capacity(active_subs.len());
            for sub in &active_subs {
                let time_offset = t - sub.start;

                let font_size = sub
                    .font_size
                    .or(config.style.font_size)
                    .unwrap_or(28.0);

                let sub_style = RenderStateStyle {
                    fontSize: font_size,
                    fontColor: sub
                        .font_color
                        .clone()
                        .or(config.style.font_color.clone())
                        .unwrap_or_else(|| "#ffffff".to_string()),
                    backgroundColor: sub
                        .background_color
                        .clone()
                        .or(config.style.background_color.clone())
                        .unwrap_or_else(|| "rgba(10, 10, 10, 0.85)".to_string()),
                    customFont: sub
                        .custom_font
                        .clone()
                        .or(config.style.custom_font.clone())
                        .unwrap_or_default(),
                    fontOpacity: sub
                        .font_opacity
                        .or(config.style.font_opacity)
                        .unwrap_or(1.0),
                    backgroundOpacity: sub
                        .background_opacity
                        .or(config.style.background_opacity)
                        .unwrap_or(0.85),
                    animationType: sub
                        .animation_type
                        .clone()
                        .or(config.style.animation_type.clone())
                        .unwrap_or_else(|| "none".to_string()),
                    animationSpeed: sub
                        .animation_speed
                        .or(config.style.animation_speed)
                        .unwrap_or(200.0),
                };

                let sub_x = sub.sub_x.or(pos.sub_x).unwrap_or(50.0);
                let sub_y = sub.sub_y.or(pos.sub_y).unwrap_or(85.0);
                let sub_width = sub.sub_width.or(pos.sub_width).unwrap_or(70.0);

                active_payload.push(RenderStateSub {
                    content: sub.content.clone(),
                    subX: sub_x,
                    subY: sub_y,
                    subWidth: sub_width,
                    timeOffset: time_offset,
                    style: sub_style,
                });
            }

            if prev_sub_id.as_ref() != Some(&sub_ids) || is_animated {
                let mut custom_font_base64 = None;
                let mut custom_font_name = None;

                if let Some(font_file) = config.style.custom_font.as_ref() {
                    let font_path = app.path().app_data_dir()
                        .map_err(|e| format!("failed to get app data dir: {e}"))?
                        .join("fonts")
                        .join(font_file);

                    if font_path.exists() {
                        let bytes = fs::read(&font_path)
                            .map_err(|e| format!("failed to read font {:?}: {}", font_path, e))?;
                        custom_font_base64 = Some(base64::engine::general_purpose::STANDARD.encode(bytes));
                        custom_font_name = Some(font_file.clone());
                    }
                }

                let state = RenderState {
                    subtitles: active_payload,
                    scaleFactor: scale_factor,
                    customFontBase64: custom_font_base64,
                    customFontName: custom_font_name,
                };

                let js = format!("window.setRenderState({});", serde_json::to_string(&state).map_err(|e| e.to_string())?);
                tab.evaluate(&js, false)
                    .map_err(|e| format!("failed to set render state: {}", e))?;
            }

            if is_animated {
                thread::sleep(Duration::from_millis(frame_interval_ms));
                active_frame_bytes = Some(
                    tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)
                        .map_err(|e| format!("failed to capture animated frame: {}", e))?,
                );
            } else if prev_sub_id.as_ref() != Some(&sub_ids) || active_frame_bytes.is_none() {
                active_frame_bytes = Some(
                    tab.capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)
                        .map_err(|e| format!("failed to capture static frame: {}", e))?,
                );
            }

            prev_sub_id = Some(sub_ids);
            active_frame_bytes.clone().unwrap()
        };

        ffmpeg_stdin
            .write_all(&frame_bytes)
            .map_err(|e| format!("failed to write frame to ffmpeg: {}", e))?;

        let pct = (frame_idx as f64 / total_frames as f64) * 100.0;
        let _ = app_clone.emit("render-progress", pct);
        eprintln!("PROGRESS:{pct:.1}");
    }

    drop(ffmpeg_stdin);

    let status = ffmpeg
        .wait()
        .map_err(|e| format!("failed to wait for ffmpeg: {}", e))?;

    let _ = stdout_handle.join();
    let _ = stderr_handle.join();

    if !status.success() {
        let log = stderr_log.lock().unwrap().clone();
        return Err(format!("ffmpeg failed: {}", log));
    }

    let _ = app.emit("render-progress", 100.0);

    Ok(PyResponse {
        status: "ok".to_string(),
        message: serde_json::Value::String("Render completed successfully".to_string()),
    })
}

#[tauri::command]
pub async fn run_render(app: AppHandle, config_json: String) -> Result<PyResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let config: RenderConfig =
            serde_json::from_str(&config_json).map_err(|e| format!("bad config json: {e}"))?;
        render_video(&app, config)
    })
    .await
    .map_err(|e| e.to_string())?
}
