mod render;
mod subtitle;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::net::TcpListener;

static CURRENT_VIDEO_PATH: std::sync::OnceLock<Mutex<Option<String>>> = std::sync::OnceLock::new();
static SERVER_PORT: std::sync::OnceLock<u16> = std::sync::OnceLock::new();

fn get_current_video_path() -> &'static Mutex<Option<String>> {
    CURRENT_VIDEO_PATH.get_or_init(|| Mutex::new(None))
}

fn get_server_port() -> u16 {
    *SERVER_PORT.get().unwrap_or(&0)
}

async fn start_http_server() {
    let listener = match TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind local HTTP server: {}", e);
            return;
        }
    };

    if let Ok(addr) = listener.local_addr() {
        let _ = SERVER_PORT.set(addr.port());
        println!("Local streaming HTTP server listening on http://{}", addr);
    }

    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                tokio::spawn(async move {
                    handle_client(socket).await;
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

async fn handle_client(mut socket: tokio::net::TcpStream) {
    let mut buf = [0u8; 1024];
    let n = match socket.read(&mut buf).await {
        Ok(0) | Err(_) => return,
        Ok(bytes) => bytes,
    };

    let request_str = String::from_utf8_lossy(&buf[..n]);

    let request_path = request_str
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    if request_path != "/video" {
        let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let _ = socket.write_all(response.as_bytes()).await;
        return;
    }

    // Parse HTTP Range header if present (standard for video players seeking/streaming)
    let mut range_start = 0;
    let mut range_end = None;
    let mut is_range = false;

    for line in request_str.lines() {
        if line.to_lowercase().starts_with("range:") {
            if let Some(pos) = line.find('=') {
                let range_val = &line[pos + 1..];
                let parts: Vec<&str> = range_val.split('-').collect();
                if !parts.is_empty() {
                    if let Ok(start) = parts[0].trim().parse::<u64>() {
                        range_start = start;
                        is_range = true;
                    }
                    if parts.len() > 1 && !parts[1].trim().is_empty() {
                        if let Ok(end) = parts[1].trim().parse::<u64>() {
                            range_end = Some(end);
                        }
                    }
                }
            }
        }
    }

    let path_opt = get_current_video_path().lock().unwrap().clone();
    let file_path = match path_opt {
        Some(p) => PathBuf::from(p),
        None => {
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            let _ = socket.write_all(response.as_bytes()).await;
            return;
        }
    };

    let mut file = match File::open(&file_path).await {
        Ok(f) => f,
        Err(_) => {
            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            let _ = socket.write_all(response.as_bytes()).await;
            return;
        }
    };

    let metadata = match file.metadata().await {
        Ok(m) => m,
        Err(_) => {
            let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n";
            let _ = socket.write_all(response.as_bytes()).await;
            return;
        }
    };

    let file_size = metadata.len();
    let end = range_end.unwrap_or(file_size - 1).min(file_size - 1);
    let chunk_size = if file_size == 0 {
        0
    } else {
        end - range_start + 1
    };

    if file
        .seek(std::io::SeekFrom::Start(range_start))
        .await
        .is_err()
    {
        let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n";
        let _ = socket.write_all(response.as_bytes()).await;
        return;
    }

    let mime_type = match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("ogg") | Some("ogv") => "video/ogg",
        Some("mov") => "video/quicktime",
        Some("mkv") => "video/x-matroska",
        _ => "video/mp4",
    };

    let mut headers = String::new();
    if is_range {
        headers.push_str("HTTP/1.1 206 Partial Content\r\n");
        headers.push_str(&format!(
            "Content-Range: bytes {}-{}/{}\r\n",
            range_start, end, file_size
        ));
    } else {
        headers.push_str("HTTP/1.1 200 OK\r\n");
    }
    headers.push_str("Access-Control-Allow-Origin: *\r\n");
    headers.push_str("Accept-Ranges: bytes\r\n");
    headers.push_str(&format!("Content-Type: {}\r\n", mime_type));
    headers.push_str(&format!("Content-Length: {}\r\n", chunk_size));
    headers.push_str("Connection: close\r\n\r\n");

    if socket.write_all(headers.as_bytes()).await.is_err() {
        return;
    }

    let mut remaining = chunk_size;
    let mut buffer = [0u8; 64 * 1024]; // 64KB chunks
    while remaining > 0 {
        let to_read = remaining.min(buffer.len() as u64) as usize;
        match file.read(&mut buffer[..to_read]).await {
            Ok(0) => break,
            Ok(read_bytes) => {
                if socket.write_all(&buffer[..read_bytes]).await.is_err() {
                    break;
                }
                remaining -= read_bytes as u64;
            }
            Err(_) => break,
        }
    }
}

#[tauri::command]
async fn run_python(
    app: AppHandle,
    name: String,
    count: i32,
) -> Result<subtitle::PyResponse, String> {
    subtitle::getvideo(app, name, count.max(1) as usize).await
}

#[tauri::command]
fn save_font(name: String, data: Vec<u8>) -> Result<String, String> {
    let font_dir = PathBuf::from("../backend/fonts");
    std::fs::create_dir_all(&font_dir)
        .map_err(|e| format!("Failed to create fonts directory: {}", e))?;

    let file_path = font_dir.join(&name);
    std::fs::write(&file_path, data).map_err(|e| format!("Failed to save font file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
fn get_streaming_url(path: String) -> Result<String, String> {
    let mut current_path = get_current_video_path().lock().unwrap();
    *current_path = Some(path);

    let port = get_server_port();
    if port == 0 {
        return Err("Streaming server not started".to_string());
    }

    let url = format!("http://127.0.0.1:{}/video", port);
    Ok(url)
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            tauri::async_runtime::spawn(async move {
                start_http_server().await;
            });
            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            run_python,
            render::run_render,
            save_font,
            get_streaming_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
