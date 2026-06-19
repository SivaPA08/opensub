use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};


use tauri::{AppHandle, Emitter, Manager};
use whisper_rs::{
    convert_integer_to_float_audio, FullParams, SamplingStrategy, WhisperContext,
    WhisperContextParameters,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sub {
    pub text: String,
    pub start: f32,
    pub end: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyResponse {
    pub status: String,
    pub message: serde_json::Value,
}

#[derive(Debug, Clone)]
struct WordStamp {
    word: String,
    start: f32,
    end: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgressPayload {
    pub model: String,
    pub progress: f64,
}

use crate::render::{get_ffmpeg_command, get_ffprobe_command};

fn ffprobe_duration(app: &AppHandle, filepath: &str) -> Result<f32, String> {
    let output = get_ffprobe_command(app)?
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            filepath,
        ])
        .output()
        .map_err(|e| format!("ffprobe failed: {}", e))?;

    if !output.status.success() {
        return Err("ffprobe returned a non-zero exit code".to_string());
    }

    let info: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("bad ffprobe json: {}", e))?;

    let duration = info["format"]["duration"]
        .as_str()
        .ok_or_else(|| "ffprobe did not return duration".to_string())?
        .parse::<f32>()
        .map_err(|e| format!("invalid duration: {}", e))?;

    Ok(duration)
}

fn download_model_file(app: &AppHandle, model_name: &str) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to get app data dir: {}", e))?;

    let filename = match model_name {
        "tiny" => "ggml-tiny.bin",
        "base" => "ggml-base.bin",
        "small" => "ggml-small.bin",
        "medium" => "ggml-medium.bin",
        _ => return Err(format!("Unsupported model: {}", model_name)),
    };

    let model_path = app_dir.join("models").join(filename);

    if model_path.exists() {
        return Ok(());
    }

    if let Some(parent) = model_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create model dir: {}", e))?;
    }

    let url = format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}", filename);
    let tmp_path = model_path.with_extension("bin.part");

    let mut resp = reqwest::blocking::get(&url)
        .map_err(|e| format!("failed to start model download: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "model download failed with status {}",
            resp.status()
        ));
    }

    let total = resp.content_length().unwrap_or(0);
    let mut file = fs::File::create(&tmp_path)
        .map_err(|e| format!("failed to create temp model file: {}", e))?;

    let mut downloaded: u64 = 0;
    let mut buf = [0u8; 64 * 1024];

    loop {
        let n = resp
            .read(&mut buf)
            .map_err(|e| format!("failed while downloading model: {}", e))?;

        if n == 0 {
            break;
        }

        file.write_all(&buf[..n])
            .map_err(|e| format!("failed while saving model: {}", e))?;

        downloaded += n as u64;

        if total > 0 {
            let pct = (downloaded as f64 / total as f64) * 100.0;
            let _ = app.emit("model-download-progress", DownloadProgressPayload {
                model: model_name.to_string(),
                progress: pct,
            });
            let subtitle_pct = 15.0 + (pct * 0.25);
            let _ = app.emit("subtitle-progress", subtitle_pct.min(39.9));
        }
    }

    file.flush().ok();

    fs::rename(&tmp_path, model_path)
        .map_err(|e| format!("failed to finalize model file: {}", e))?;

    Ok(())
}

fn transcribe_words(
    app: &AppHandle,
    filename: &str,
    max_words: usize,
    model_name: &str,
) -> Result<Vec<Sub>, String> {
    let max_words = max_words.max(1);

    let duration = ffprobe_duration(app, filename).unwrap_or(0.0);

    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to get app data dir: {}", e))?;

    let model_filename = match model_name {
        "tiny" => "ggml-tiny.bin",
        "base" => "ggml-base.bin",
        "small" => "ggml-small.bin",
        "medium" => "ggml-medium.bin",
        _ => return Err(format!("Unsupported model: {}", model_name)),
    };
    let model_path = app_dir.join("models").join(model_filename);

    let _ = app.emit("subtitle-progress", 5.0);
    let _ = app.emit("subtitle-progress", 15.0);

    if !model_path.exists() {
        download_model_file(app, model_name)?;
    }

    let _ = app.emit("subtitle-progress", 40.0);

    let ctx = WhisperContext::new_with_params(
        model_path
            .to_str()
            .ok_or_else(|| "invalid model path".to_string())?,
        WhisperContextParameters::default(),
    )
    .map_err(|e| format!("failed to load model: {}", e))?;

    let mut state = ctx
        .create_state()
        .map_err(|e| format!("failed to create whisper state: {}", e))?;

    let threads = std::thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(4);

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_n_threads(threads);
    params.set_translate(false);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_token_timestamps(true);

    let wav_path = {
        let tmp = std::env::temp_dir().join("opensub_audio.wav");
        let status = get_ffmpeg_command(app)?
            .args([
                "-y",
                "-i",
                filename,
                "-ar",
                "16000",
                "-ac",
                "1",
                "-c:a",
                "pcm_s16le",
            ])
            .arg(&tmp)
            .status()
            .map_err(|e| format!("ffmpeg failed: {}", e))?;

        if !status.success() {
            return Err("ffmpeg returned a non-zero exit code while extracting audio".to_string());
        }

        tmp
    };

    let reader =
        hound::WavReader::open(&wav_path).map_err(|e| format!("failed to open wav: {}", e))?;

    let samples: Vec<i16> = reader
        .into_samples::<i16>()
        .map(|s| s.map_err(|e| format!("wav sample error: {}", e)))
        .collect::<Result<Vec<_>, _>>()?;

    let mut audio = vec![0.0f32; samples.len()];
    convert_integer_to_float_audio(&samples, &mut audio)
        .map_err(|e| format!("audio conversion failed: {}", e))?;

    state
        .full(params, &audio[..])
        .map_err(|e| format!("failed to run whisper: {}", e))?;

    let num_segments = state.full_n_segments();

    let mut words: Vec<WordStamp> = Vec::new();

    for i in 0..num_segments {
        let segment = state.get_segment(i)
            .ok_or_else(|| format!("failed to get segment at index {}", i))?;
        let text = segment.to_string().trim().to_string();

        if text.is_empty() {
            continue;
        }

        let start_ts = segment.start_timestamp();
        let end_ts = segment.end_timestamp();

        if duration > 0.0 {
            let pct = ((end_ts as f32 / 100.0) / duration) * 100.0;
            let scaled_pct: f32 = 40.0 + (pct * 0.55);
            let _ = app.emit("subtitle-progress", scaled_pct.min(99.0f32));
        }

        let seg_start = start_ts as f32 / 100.0;
        let seg_end = end_ts as f32 / 100.0;
        let seg_len = (seg_end - seg_start).max(0.01);

        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let per_word = seg_len / parts.len() as f32;

        for (idx, part) in parts.iter().enumerate() {
            let w_start = seg_start + per_word * idx as f32;
            let w_end = if idx + 1 == parts.len() {
                seg_end
            } else {
                seg_start + per_word * (idx + 1) as f32
            };

            words.push(WordStamp {
                word: part.trim().to_string(),
                start: w_start,
                end: w_end,
            });
        }
    }

    let _ = fs::remove_file(&wav_path);

    let mut subs: Vec<Sub> = Vec::new();
    let mut curr_words: Vec<String> = Vec::new();
    let mut start_time: Option<f32> = None;
    let mut end_time: Option<f32> = None;

    for w in words {
        if start_time.is_none() {
            start_time = Some(w.start);
        }

        curr_words.push(w.word.clone());
        end_time = Some(w.end);

        let should_break = curr_words.len() >= max_words
            || w.word.ends_with('.')
            || w.word.ends_with('?')
            || w.word.ends_with('!');

        if should_break {
            subs.push(Sub {
                text: curr_words.join(" "),
                start: start_time.unwrap_or(0.0),
                end: end_time.unwrap_or(start_time.unwrap_or(0.0)),
            });

            curr_words.clear();
            start_time = None;
            end_time = None;
        }
    }

    if !curr_words.is_empty() {
        subs.push(Sub {
            text: curr_words.join(" "),
            start: start_time.unwrap_or(0.0),
            end: end_time.unwrap_or(start_time.unwrap_or(0.0)),
        });
    }

    let _ = app.emit("subtitle-progress", 100.0);

    Ok(subs)
}

#[tauri::command]
pub async fn getvideo(
    app: AppHandle,
    filename: String,
    max_words: usize,
    model_name: String,
) -> Result<PyResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if filename.is_empty() {
            return Err("Filename is required".to_string());
        }
        if max_words == 0 {
            return Err("max_words is required".to_string());
        }

        let subs = transcribe_words(&app, &filename, max_words, &model_name)?;
        let message = serde_json::to_value(subs).map_err(|e| e.to_string())?;

        Ok(PyResponse {
            status: "ok".to_string(),
            message,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn check_models_status(app: AppHandle) -> Result<serde_json::Value, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to get app data dir: {}", e))?;

    let models_dir = app_dir.join("models");
    let mut status = serde_json::Map::new();

    for model in &["tiny", "base", "small", "medium"] {
        let filename = match *model {
            "tiny" => "ggml-tiny.bin",
            "base" => "ggml-base.bin",
            "small" => "ggml-small.bin",
            "medium" => "ggml-medium.bin",
            _ => continue,
        };
        let path = models_dir.join(filename);
        status.insert(model.to_string(), serde_json::Value::Bool(path.exists()));
    }

    Ok(serde_json::Value::Object(status))
}

#[tauri::command]
pub async fn download_model(app: AppHandle, model_name: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        download_model_file(&app, &model_name)
    })
    .await
    .map_err(|e| e.to_string())?
}
