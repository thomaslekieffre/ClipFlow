use crate::recording::manager;
use crate::state::AppState;
use crate::types::{
    Annotation, AudioDevice, AudioSource, Clip, ExportFormat, ExportQuality,
    RecordingState, Region, Subtitle, TransitionType, WindowInfo,
};
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
    let enabled = state.audio_source != AudioSource::None;
    if enabled {
        state.audio_source = AudioSource::None;
    } else {
        state.audio_source = AudioSource::System;
    }
    Ok(state.audio_source != AudioSource::None)
}

#[tauri::command]
pub fn set_audio_source(state: State<'_, Mutex<AppState>>, source: AudioSource) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.audio_source = source;
    Ok(())
}

#[tauri::command]
pub fn get_audio_source(state: State<'_, Mutex<AppState>>) -> Result<AudioSource, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.audio_source)
}

#[tauri::command]
pub fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    crate::capture::audio::list_audio_devices()
}

#[tauri::command]
pub fn set_selected_mic(state: State<'_, Mutex<AppState>>, device_name: Option<String>) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.selected_mic = device_name;
    Ok(())
}

#[tauri::command]
pub fn get_selected_mic(state: State<'_, Mutex<AppState>>) -> Result<Option<String>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.selected_mic.clone())
}

#[tauri::command]
pub fn set_audio_volumes(
    state: State<'_, Mutex<AppState>>,
    system_volume: f32,
    mic_volume: f32,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.system_volume = system_volume.clamp(0.0, 2.0);
    state.mic_volume = mic_volume.clamp(0.0, 2.0);
    Ok(())
}

#[tauri::command]
pub fn get_audio_volumes(state: State<'_, Mutex<AppState>>) -> Result<(f32, f32), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok((state.system_volume, state.mic_volume))
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

    // Clean up associated data
    state.annotations.remove(&clip_id);
    state.clip_keystrokes.remove(&clip_id);
    state.clip_cursor_positions.remove(&clip_id);

    Ok(())
}

#[tauri::command]
pub fn set_transition(
    state: State<'_, Mutex<AppState>>,
    index: usize,
    transition_type: TransitionType,
    duration_s: Option<f64>,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    if index >= state.transitions.len() {
        return Err(format!("Transition index {} out of bounds", index));
    }
    state.transitions[index].transition_type = transition_type;
    if let Some(dur) = duration_s {
        state.transitions[index].duration_s = dur.clamp(0.1, 5.0);
    }
    Ok(())
}

#[tauri::command]
pub fn set_all_transitions(
    state: State<'_, Mutex<AppState>>,
    transition_type: TransitionType,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    for t in state.transitions.iter_mut() {
        t.transition_type = transition_type;
    }
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
pub fn start_recording(state: State<'_, Mutex<AppState>>, app: AppHandle) -> Result<(), String> {
    manager::start(&state, &app)
}

#[tauri::command]
pub async fn stop_recording(state: State<'_, Mutex<AppState>>) -> Result<Clip, String> {
    manager::stop(&state).await
}

#[tauri::command]
pub async fn pause_recording(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    manager::pause(&state).await
}

#[tauri::command]
pub fn resume_recording(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    manager::resume(&state)
}

#[tauri::command]
pub fn cancel_recording(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    manager::cancel(&state)
}

#[tauri::command]
pub fn get_recording_duration_ms(state: State<'_, Mutex<AppState>>) -> Result<u64, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    let current = match state.recording_start {
        Some(start) => start.elapsed().as_millis() as u64,
        None => 0,
    };
    Ok(state.pause_accumulated_ms + current)
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
    let (clips, transitions, clip_keystrokes, subtitles, clip_annotations, clip_cursor_positions, system_volume, mic_volume) = {
        let s = state.lock().map_err(|e| e.to_string())?;
        (s.clips.clone(), s.transitions.clone(), s.clip_keystrokes.clone(), s.subtitles.clone(), s.annotations.clone(), s.clip_cursor_positions.clone(), s.system_volume, s.mic_volume)
    };

    if clips.is_empty() {
        return Err("Aucun clip à exporter".into());
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
            crate::export::encoder::export_mp4(&clips, &transitions, &output_path, &app, watermark, &quality, &clip_keystrokes, &subtitles, &clip_annotations, &clip_cursor_positions, system_volume, mic_volume)
                .await
                .map_err(|e| {
                    eprintln!("[export_video] FAILED: {}", e);
                    format!("Export échoué : {}", e)
                })?;
        }
        ExportFormat::Gif => {
            crate::export::encoder::export_gif(&clips, &transitions, &output_path, &app, watermark, &quality, &clip_keystrokes, &subtitles, &clip_annotations, &clip_cursor_positions, system_volume, mic_volume)
                .await
                .map_err(|e| {
                    eprintln!("[export_video] FAILED: {}", e);
                    format!("Export échoué : {}", e)
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
    let (clips, transitions, clip_keystrokes, subtitles, clip_annotations, clip_cursor_positions) = {
        let s = state.lock().map_err(|e| e.to_string())?;
        (s.clips.clone(), s.transitions.clone(), s.clip_keystrokes.clone(), s.subtitles.clone(), s.annotations.clone(), s.clip_cursor_positions.clone())
    };

    if clips.is_empty() {
        return Err("Aucun clip à prévisualiser".into());
    }

    // Use temp directory for preview
    let preview_dir = std::env::temp_dir().join("clipflow_preview");
    std::fs::create_dir_all(&preview_dir).map_err(|e| e.to_string())?;
    let preview_path = preview_dir.join("preview.mp4");

    eprintln!("[preview_video] {} clips, output: {:?}", clips.len(), preview_path);

    crate::export::encoder::preview_mp4(
        &clips, &transitions, &preview_path, &app,
        &clip_keystrokes, &subtitles, &clip_annotations, &clip_cursor_positions,
    )
        .await
        .map_err(|e| {
            eprintln!("[preview_video] FAILED: {}", e);
            format!("Prévisualisation échouée : {}", e)
        })?;

    Ok(preview_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn ensure_ffmpeg() -> Result<String, String> {
    let ffmpeg = crate::ffmpeg_bin();
    if ffmpeg.exists() {
        return Ok(ffmpeg.to_string_lossy().to_string());
    }
    // Download FFmpeg to our AppData/Local/ClipFlow/ directory
    let download_dir = dirs::data_local_dir()
        .ok_or("Cannot find local data directory")?
        .join("ClipFlow");
    std::fs::create_dir_all(&download_dir).map_err(|e| e.to_string())?;

    let dest = download_dir.clone();
    tokio::task::spawn_blocking(move || {
        let url = ffmpeg_sidecar::download::ffmpeg_download_url()
            .map_err(|e| format!("Failed to get download URL: {}", e))?;
        let archive = ffmpeg_sidecar::download::download_ffmpeg_package(url, &dest)
            .map_err(|e| format!("Failed to download FFmpeg: {}", e))?;
        ffmpeg_sidecar::download::unpack_ffmpeg(&archive, &dest)
            .map_err(|e| format!("Failed to unpack FFmpeg: {}", e))?;
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("FFmpeg download task failed: {}", e))?
    .map_err(|e| format!("FFmpeg download failed: {}", e))?;

    let ffmpeg = crate::ffmpeg_bin();
    if ffmpeg.exists() {
        Ok(ffmpeg.to_string_lossy().to_string())
    } else {
        Err("FFmpeg downloaded but binary not found".into())
    }
}

// Window snapping
#[tauri::command]
pub fn get_visible_windows() -> Result<Vec<WindowInfo>, String> {
    crate::region::selector::enumerate_visible_windows()
}

// Countdown
#[tauri::command]
pub fn set_countdown(state: State<'_, Mutex<AppState>>, seconds: u32) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.countdown_seconds = seconds;
    Ok(())
}

#[tauri::command]
pub fn get_countdown(state: State<'_, Mutex<AppState>>) -> Result<u32, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.countdown_seconds)
}

// Annotations
#[tauri::command]
pub fn set_clip_annotations(
    state: State<'_, Mutex<AppState>>,
    clip_id: String,
    annotations: Vec<Annotation>,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.annotations.insert(clip_id, annotations);
    Ok(())
}

#[tauri::command]
pub fn get_clip_annotations(
    state: State<'_, Mutex<AppState>>,
    clip_id: String,
) -> Result<Vec<Annotation>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.annotations.get(&clip_id).cloned().unwrap_or_default())
}

// Subtitles
#[tauri::command]
pub fn set_subtitles(
    state: State<'_, Mutex<AppState>>,
    subtitles: Vec<Subtitle>,
) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.subtitles = subtitles;
    Ok(())
}

#[tauri::command]
pub fn get_subtitles(state: State<'_, Mutex<AppState>>) -> Result<Vec<Subtitle>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.subtitles.clone())
}

// Keystroke toggle
#[tauri::command]
pub fn toggle_keystroke_display(state: State<'_, Mutex<AppState>>) -> Result<bool, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.keystroke_enabled = !state.keystroke_enabled;
    Ok(state.keystroke_enabled)
}

#[tauri::command]
pub fn get_keystroke_enabled(state: State<'_, Mutex<AppState>>) -> Result<bool, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.keystroke_enabled)
}

// Cursor zoom toggle
#[tauri::command]
pub fn toggle_cursor_zoom(state: State<'_, Mutex<AppState>>) -> Result<bool, String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.cursor_zoom_enabled = !state.cursor_zoom_enabled;
    Ok(state.cursor_zoom_enabled)
}

#[tauri::command]
pub fn get_cursor_zoom_enabled(state: State<'_, Mutex<AppState>>) -> Result<bool, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.cursor_zoom_enabled)
}

// Clipboard export
#[tauri::command]
pub fn copy_file_to_clipboard(path: String) -> Result<(), String> {
    // Use arboard to copy file path to clipboard
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(&path).map_err(|e| e.to_string())?;
    Ok(())
}

// Project commands
#[tauri::command]
pub async fn save_project(
    state: State<'_, Mutex<AppState>>,
    name: String,
) -> Result<String, String> {
    let project_data = {
        let s = state.lock().map_err(|e| e.to_string())?;
        (
            s.clips.clone(),
            s.transitions.clone(),
            s.audio_source,
            s.annotations.clone(),
            s.subtitles.clone(),
            s.current_project_id.clone(),
        )
    };

    let project_id = crate::project::save_project(
        project_data.5,
        &name,
        &project_data.0,
        &project_data.1,
        project_data.2,
        &project_data.3,
        &project_data.4,
    )?;

    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.current_project_id = Some(project_id.clone());
    }

    Ok(project_id)
}

#[tauri::command]
pub fn load_project(
    state: State<'_, Mutex<AppState>>,
    project_id: String,
) -> Result<(), String> {
    let project = crate::project::load_project(&project_id)?;

    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.clips = project.clips;
    s.transitions = project.transitions;
    s.audio_source = project.settings.audio_source;
    s.annotations = project.annotations;
    s.subtitles = project.subtitles;
    s.current_project_id = Some(project_id);

    Ok(())
}

#[tauri::command]
pub fn list_projects() -> Result<Vec<crate::types::ProjectSummary>, String> {
    crate::project::list_projects()
}

#[tauri::command]
pub fn delete_project(project_id: String) -> Result<(), String> {
    crate::project::delete_project(&project_id)
}
