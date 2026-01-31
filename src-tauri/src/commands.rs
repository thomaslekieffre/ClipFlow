use crate::recording::manager;
use crate::state::AppState;
use crate::types::{Clip, ExportFormat, ExportQuality, RecordingState, Region, TransitionType};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_notification::NotificationExt;

#[tauri::command]
pub fn get_recording_state(state: State<'_, Mutex<AppState>>) -> Result<RecordingState, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.recording_state)
}

#[tauri::command]
pub fn set_capture_region(state: State<'_, Mutex<AppState>>, region: Region) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.current_region = Some(region);
    Ok(())
}

#[tauri::command]
pub fn toggle_audio(state: State<'_, Mutex<AppState>>) -> Result<bool, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.audio_enabled = !state.audio_enabled;
    Ok(state.audio_enabled)
}

#[tauri::command]
pub fn get_clips(state: State<'_, Mutex<AppState>>) -> Result<Vec<Clip>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.clips.clone())
}

#[tauri::command]
pub fn reorder_clips(
    state: State<'_, Mutex<AppState>>,
    clip_ids: Vec<String>,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let mut reordered = Vec::with_capacity(clip_ids.len());
    for id in &clip_ids {
        let clip = state
            .clips
            .iter()
            .find(|c| &c.id == id)
            .cloned()
            .ok_or_else(|| format!("Clip not found: {}", id))?;
        reordered.push(clip);
    }
    state.clips = reordered;
    Ok(())
}

#[tauri::command]
pub fn delete_clip(state: State<'_, Mutex<AppState>>, clip_id: String) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;

    // Delete clip files from disk
    if let Some(clip) = state.clips.iter().find(|c| c.id == clip_id) {
        let _ = std::fs::remove_file(&clip.path);
        if let Some(ref thumb) = clip.thumbnail_path {
            let _ = std::fs::remove_file(thumb);
        }
    }

    state.clips.retain(|c| c.id != clip_id);
    let max_transitions = state.clips.len().saturating_sub(1);
    state.transitions.truncate(max_transitions);
    Ok(())
}

#[tauri::command]
pub fn set_transition(
    state: State<'_, Mutex<AppState>>,
    index: usize,
    transition_type: TransitionType,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    if index >= state.transitions.len() {
        return Err(format!("Transition index {} out of bounds", index));
    }
    state.transitions[index].transition_type = transition_type;
    Ok(())
}

#[tauri::command]
pub fn set_clip_trim(
    state: State<'_, Mutex<AppState>>,
    clip_id: String,
    trim_start_ms: u64,
    trim_end_ms: u64,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let clip = state.clips.iter_mut().find(|c| c.id == clip_id)
        .ok_or_else(|| format!("Clip not found: {}", clip_id))?;
    clip.trim_start_ms = trim_start_ms;
    clip.trim_end_ms = trim_end_ms;
    Ok(())
}

#[tauri::command]
pub fn start_recording(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    manager::start(&state)
}

#[tauri::command]
pub async fn stop_recording(state: State<'_, Mutex<AppState>>) -> Result<Clip, String> {
    manager::stop(&state).await
}

#[tauri::command]
pub fn cancel_recording(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    manager::cancel(&state)
}

#[tauri::command]
pub fn get_recording_duration_ms(state: State<'_, Mutex<AppState>>) -> Result<u64, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    match state.recording_start {
        Some(start) => Ok(start.elapsed().as_millis() as u64),
        None => Ok(0),
    }
}

#[tauri::command]
pub fn get_thumbnail_base64(
    state: State<'_, Mutex<AppState>>,
    clip_id: String,
) -> Result<Option<String>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    let clip = state.clips.iter().find(|c| c.id == clip_id);
    if let Some(clip) = clip {
        if let Some(ref thumb_path) = clip.thumbnail_path {
            if thumb_path.exists() {
                let bytes = std::fs::read(thumb_path).map_err(|e| e.to_string())?;
                use base64::Engine;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                return Ok(Some(format!("data:image/png;base64,{}", b64)));
            }
        }
    }
    Ok(None)
}

#[tauri::command]
pub fn get_transitions(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<crate::types::Transition>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.transitions.clone())
}

#[tauri::command]
pub async fn open_region_selector(app: AppHandle) -> Result<(), String> {
    // Close existing overlay if any
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.close();
    }

    // Calculate bounding box across all monitors
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    let mut min_x: i32 = 0;
    let mut min_y: i32 = 0;
    let mut max_x: i32 = 1920;
    let mut max_y: i32 = 1080;

    if !monitors.is_empty() {
        min_x = i32::MAX;
        min_y = i32::MAX;
        max_x = i32::MIN;
        max_y = i32::MIN;
        for m in &monitors {
            let pos = m.position();
            let size = m.size();
            min_x = min_x.min(pos.x);
            min_y = min_y.min(pos.y);
            max_x = max_x.max(pos.x + size.width as i32);
            max_y = max_y.max(pos.y + size.height as i32);
        }
    }

    let width = (max_x - min_x) as f64;
    let height = (max_y - min_y) as f64;

    WebviewWindowBuilder::new(&app, "overlay", WebviewUrl::App("/overlay".into()))
        .title("Region Selector")
        .position(min_x as f64, min_y as f64)
        .inner_size(width, height)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(true)
        .build()
        .map_err(|e| format!("Failed to open overlay: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_monitors_info(app: AppHandle) -> Result<Vec<MonitorInfo>, String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    Ok(monitors.iter().map(|m| {
        let pos = m.position();
        let size = m.size();
        MonitorInfo {
            x: pos.x,
            y: pos.y,
            width: size.width,
            height: size.height,
        }
    }).collect())
}

#[derive(serde::Serialize)]
pub struct MonitorInfo {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[tauri::command]
pub async fn close_region_selector(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("overlay") {
        w.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn export_video(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    watermark: bool,
    format: ExportFormat,
    quality: ExportQuality,
) -> Result<String, String> {
    let (clips, transitions) = {
        let s = state.lock().map_err(|e| e.to_string())?;
        (s.clips.clone(), s.transitions.clone())
    };

    if clips.is_empty() {
        return Err("No clips to export".into());
    }

    eprintln!("[export_video] {} clips, {} transitions, watermark={}, format={:?}, quality={:?}", clips.len(), transitions.len(), watermark, format, quality);
    for (i, clip) in clips.iter().enumerate() {
        eprintln!("[export_video] Clip {}: {:?} ({}ms, trim {}..{})", i, clip.path, clip.duration_ms, clip.trim_start_ms, clip.trim_end_ms);
    }

    // Create output directory
    let output_dir = dirs::video_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Videos"))
        .join("ClipFlow");
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    // Generate filename with timestamp
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let ext = match format {
        ExportFormat::Mp4 => "mp4",
        ExportFormat::Gif => "gif",
    };
    let output_path = output_dir.join(format!("recording_{}.{}", timestamp, ext));
    eprintln!("[export_video] Output: {:?}", output_path);

    // Run export
    match format {
        ExportFormat::Mp4 => {
            crate::export::encoder::export_mp4(&clips, &transitions, &output_path, &app, watermark, &quality)
                .await
                .map_err(|e| {
                    eprintln!("[export_video] FAILED: {}", e);
                    format!("Export failed: {}", e)
                })?;
        }
        ExportFormat::Gif => {
            crate::export::encoder::export_gif(&clips, &transitions, &output_path, &app, watermark, &quality)
                .await
                .map_err(|e| {
                    eprintln!("[export_video] FAILED: {}", e);
                    format!("Export failed: {}", e)
                })?;
        }
    }

    // Notify user
    let filename = output_path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let _ = app.notification()
        .builder()
        .title("ClipFlow")
        .body(format!("Export terminé : {}", filename))
        .show();

    // Open output folder
    eprintln!("[export_video] Opening folder: {:?}", output_dir);
    let _ = opener::open(&output_dir);

    Ok(output_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn preview_video(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
) -> Result<String, String> {
    let (clips, transitions) = {
        let s = state.lock().map_err(|e| e.to_string())?;
        (s.clips.clone(), s.transitions.clone())
    };

    if clips.is_empty() {
        return Err("No clips to preview".into());
    }

    // Use temp directory for preview
    let preview_dir = std::env::temp_dir().join("clipflow_preview");
    std::fs::create_dir_all(&preview_dir).map_err(|e| e.to_string())?;
    let preview_path = preview_dir.join("preview.mp4");

    eprintln!("[preview_video] {} clips, output: {:?}", clips.len(), preview_path);

    crate::export::encoder::preview_mp4(&clips, &transitions, &preview_path, &app)
        .await
        .map_err(|e| {
            eprintln!("[preview_video] FAILED: {}", e);
            format!("Preview failed: {}", e)
        })?;

    Ok(preview_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn ensure_ffmpeg() -> Result<String, String> {
    // Check if FFmpeg is already available
    let ffmpeg = ffmpeg_sidecar::paths::ffmpeg_path();
    if ffmpeg.exists() {
        return Ok(ffmpeg.to_string_lossy().to_string());
    }
    // Auto-download FFmpeg
    ffmpeg_sidecar::download::auto_download()
        .map_err(|e| format!("Failed to download FFmpeg: {}", e))?;
    Ok(ffmpeg.to_string_lossy().to_string())
}
