use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PyResponse {
    status: String,
    message: serde_json::Value,
}

#[tauri::command]
fn run_python(name: String, count: i32) -> Result<PyResponse, String> {
    let output = Command::new("python3")
        .arg("../backend/main.py")
        .arg(name)
        .arg(count.to_string())
        .output()
        .map_err(|e| e.to_string())?;

    let text = String::from_utf8_lossy(&output.stdout);

    let res: PyResponse = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    Ok(res)
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, run_python])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
