use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Serialize, Deserialize)]
pub struct PyResponse {
    pub status: String,
    pub message: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    pub container_width: Option<u32>,
    pub container_height: Option<u32>,
}

fn ass_time(t: f32) -> String {
    let t = t.max(0.0);
    let h = (t / 3600.0).floor() as u32;
    let m = ((t % 3600.0) / 60.0).floor() as u32;
    let s = (t % 60.0).floor() as u32;
    let cs = ((t - t.floor()) * 100.0).round() as u32;
    format!("{h}:{m:02}:{s:02}.{cs:02}")
}

fn escape_ass_text(s: &str) -> String {
    s.replace('\n', r"\N")
        .replace('{', r"\{")
        .replace('}', r"\}")
}

fn css_hex_to_ass_color(s: &str) -> String {
    let hex = s.trim().trim_start_matches('#');
    if hex.len() >= 6 {
        let r = &hex[0..2];
        let g = &hex[2..4];
        let b = &hex[4..6];
        format!("&H{}{}{}&", b, g, r)
    } else {
        "&HFFFFFF&".to_string()
    }
}

fn ffprobe_info(path: &str) -> Result<(u32, u32, f32), String> {
    let out = Command::new("ffprobe")
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
            if let Some(d) = stream["duration"].as_str() {
                duration = d.parse::<f32>().unwrap_or(duration);
            }
            break;
        }
    }

    Ok((width, height, duration))
}

fn build_ass(config: &RenderConfig, width: u32, height: u32, ass_path: &Path) -> Result<(), String> {
    let mut ass = String::new();

    let font_name = config
        .style
        .custom_font
        .clone()
        .unwrap_or_else(|| "Arial".to_string());

    ass.push_str("[Script Info]\n");
    ass.push_str("ScriptType: v4.00+\n");
    ass.push_str(&format!("PlayResX: {width}\n"));
    ass.push_str(&format!("PlayResY: {height}\n"));
    ass.push_str("WrapStyle: 2\n");
    ass.push_str("ScaledBorderAndShadow: yes\n");
    ass.push_str("\n[V4+ Styles]\n");
    ass.push_str("Format: Name,Fontname,Fontsize,PrimaryColour,SecondaryColour,OutlineColour,BackColour,Bold,Italic,Underline,StrikeOut,ScaleX,ScaleY,Spacing,Angle,BorderStyle,Outline,Shadow,Alignment,MarginL,MarginR,MarginV,Encoding\n");

    let font_size = config.style.font_size.unwrap_or(28.0);
    let font_color = css_hex_to_ass_color(
        config
            .style
            .font_color
            .as_deref()
            .unwrap_or("#ffffff"),
    );

    ass.push_str(&format!(
        "Style: Default,{font_name},{font_size}, {font_color}, &H000000&, &H000000&, &H000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n"
    ));

    ass.push_str("\n[Events]\n");
    ass.push_str("Format: Layer,Start,End,Style,Name,MarginL,MarginR,MarginV,Effect,Text\n");

    let pos = config.position.clone().unwrap_or(RenderPosition {
        sub_x: Some(50.0),
        sub_y: Some(85.0),
        sub_width: Some(70.0),
    });

    let base_x = pos.sub_x.unwrap_or(50.0) / 100.0 * width as f32;
    let base_y = pos.sub_y.unwrap_or(85.0) / 100.0 * height as f32;
    let scale_factor = if let Some(v) = &config.video_info {
        if let Some(ch) = v.container_height {
            if ch > 0 {
                height as f32 / ch as f32
            } else {
                1.0
            }
        } else {
            1.0
        }
    } else {
        1.0
    };

    for sub in &config.subtitles {
        let fs = sub
            .font_size
            .or(config.style.font_size)
            .unwrap_or(28.0)
            * scale_factor;

        let color = css_hex_to_ass_color(
            sub.font_color
                .as_deref()
                .or(config.style.font_color.as_deref())
                .unwrap_or("#ffffff"),
        );

        let start = ass_time(sub.start);
        let end = ass_time(sub.end);
        let text = escape_ass_text(&sub.content);

        let x = sub.sub_x.unwrap_or(pos.sub_x.unwrap_or(50.0)) / 100.0 * width as f32;
        let y = sub.sub_y.unwrap_or(pos.sub_y.unwrap_or(85.0)) / 100.0 * height as f32;

        ass.push_str(&format!(
            "Dialogue: 0,{start},{end},Default,,0,0,0,,{{\\an2\\pos({:.0},{:.0})\\fs{:.0}\\c{color}\\bord2\\shad0}}{text}\n",
            x,
            y,
            fs.max(8.0)
        ));
    }

    fs::write(ass_path, ass).map_err(|e| format!("failed to write ass file: {e}"))?;
    Ok(())
}

fn escape_ffmpeg_filter_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', r"\\")
        .replace(':', r"\:")
        .replace('\'', r"\'")
}

fn render_video(app: &AppHandle, config: RenderConfig) -> Result<PyResponse, String> {
    let (width, height, duration) = ffprobe_info(&config.input_path)?;

    let temp_dir = std::env::temp_dir();
    let ass_path = temp_dir.join("opensub_render.ass");

    build_ass(&config, width, height, &ass_path)?;

    let vf = format!("subtitles='{}'", escape_ffmpeg_filter_path(&ass_path));

    let mut child = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            &config.input_path,
            "-vf",
            &vf,
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-map",
            "0:v:0",
            "-map",
            "0:a?",
            "-progress",
            "pipe:1",
            "-nostats",
            &config.output_path,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn ffmpeg: {e}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "failed to capture ffmpeg stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "failed to capture ffmpeg stderr".to_string())?;

    let app_clone = app.clone();
    let progress_handle = std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            if let Some(v) = line.strip_prefix("out_time_ms=") {
                if let Ok(ms) = v.trim().parse::<u64>() {
                    if duration > 0.0 {
                        let sec = ms as f64 / 1_000_000.0;
                        let pct = ((sec / duration as f64) * 100.0).min(99.0);
                        let _ = app_clone.emit("render-progress", pct);
                    }
                }
            }
        }
    });

    let err_buf = Arc::new(Mutex::new(String::new()));
    let err_buf_clone = err_buf.clone();

    let stderr_handle = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            let mut s = err_buf_clone.lock().unwrap();
            s.push_str(&line);
            s.push('\n');
        }
    });

    let status = child
        .wait()
        .map_err(|e| format!("failed to wait for ffmpeg: {e}"))?;

    let _ = progress_handle.join();
    let _ = stderr_handle.join();
    let _ = fs::remove_file(&ass_path);

    if !status.success() {
        let log = err_buf.lock().unwrap().clone();
        return Err(format!("ffmpeg failed: {log}"));
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
