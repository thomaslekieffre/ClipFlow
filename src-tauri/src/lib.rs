mod capture;
mod commands;
mod export;
mod hotkeys;
mod project;
mod recording;
mod region;
mod state;
mod types;

use state::AppState;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

/// Windows flag to prevent spawning a visible console window.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Return the path to FFmpeg, checking our AppData location first,
/// then the sidecar location, then PATH fallback.
pub fn ffmpeg_bin() -> PathBuf {
    // 1. Check our custom AppData/Local/ClipFlow/ location
    if let Some(data_dir) = dirs::data_local_dir() {
        let custom = data_dir.join("ClipFlow").join(if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" });
        if custom.exists() {
            return custom;
        }
    }
    // 2. Fall back to sidecar location (next to exe)
    ffmpeg_sidecar::paths::ffmpeg_path()
}

/// Return the path to FFprobe (same directory as FFmpeg).
pub fn ffprobe_bin() -> PathBuf {
    let name = if cfg!(windows) { "ffprobe.exe" } else { "ffprobe" };
    ffmpeg_bin().with_file_name(name)
}

/// Create an async FFmpeg command with hidden console window on Windows.
pub fn ffmpeg_command() -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(ffmpeg_bin());
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// Create a sync FFmpeg command with hidden console window on Windows.
pub fn ffmpeg_command_sync() -> std::process::Command {
    let mut cmd = std::process::Command::new(ffmpeg_bin());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// Create an async FFprobe command with hidden console window on Windows.
pub fn ffprobe_command() -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(ffprobe_bin());
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set FFMPEG_DOWNLOAD_DIR so ffmpeg-sidecar downloads to a writable location
    // (not next to the exe, which may be in Program Files)
    if let Some(data_dir) = dirs::data_local_dir() {
        let ffmpeg_dir = data_dir.join("ClipFlow");
        let _ = std::fs::create_dir_all(&ffmpeg_dir);
        std::env::set_var("FFMPEG_DOWNLOAD_DIR", &ffmpeg_dir);
    }

    tauri::Builder::default()
        .setup(|app| {
            // Set window icon (needed during dev mode)
            let png_bytes = include_bytes!("../icons/icon.png");
            if let Ok(img) = image::load_from_memory(png_bytes) {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                let icon = tauri::image::Image::new_owned(rgba.into_raw(), w, h);
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.set_icon(icon);
                }
            }

            // Cleanup old temp files on startup (> 24h)
            let temp_dir = dirs::data_local_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("ClipFlow")
                .join("temp");
            if temp_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&temp_dir) {
                    let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(24 * 3600);
                    for entry in entries.flatten() {
                        if let Ok(meta) = entry.metadata() {
                            if let Ok(modified) = meta.modified() {
                                if modified < cutoff {
                                    let _ = std::fs::remove_file(entry.path());
                                }
                            }
                        }
                    }
                }
            }

            // Cleanup preview temp files
            let preview_dir = std::env::temp_dir().join("clipflow_preview");
            if preview_dir.exists() {
                let _ = std::fs::remove_dir_all(&preview_dir);
            }

            Ok(())
        })
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts(["F9", "F10", "Escape"])
                .unwrap_or_else(|e| {
                    eprintln!("[init] Failed to register shortcuts: {}", e);
                    tauri_plugin_global_shortcut::Builder::new()
                })
                .with_handler(hotkeys::handler)
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            commands::get_recording_state,
            commands::set_capture_region,
            commands::toggle_audio,
            commands::set_audio_source,
            commands::get_audio_source,
            commands::get_audio_devices,
            commands::get_clips,
            commands::reorder_clips,
            commands::delete_clip,
            commands::set_transition,
            commands::set_all_transitions,
            commands::set_clip_trim,
            commands::start_recording,
            commands::stop_recording,
            commands::pause_recording,
            commands::resume_recording,
            commands::cancel_recording,
            commands::get_recording_duration_ms,
            commands::get_thumbnail_base64,
            commands::get_transitions,
            commands::open_region_selector,
            commands::close_region_selector,
            commands::get_monitors_info,
            commands::export_video,
            commands::preview_video,
            commands::ensure_ffmpeg,
            commands::get_visible_windows,
            commands::set_countdown,
            commands::get_countdown,
            commands::set_clip_annotations,
            commands::get_clip_annotations,
            commands::set_subtitles,
            commands::get_subtitles,
            commands::toggle_keystroke_display,
            commands::get_keystroke_enabled,
            commands::toggle_cursor_zoom,
            commands::get_cursor_zoom_enabled,
            commands::copy_file_to_clipboard,
            commands::save_project,
            commands::load_project,
            commands::list_projects,
            commands::delete_project,
            commands::set_selected_mic,
            commands::get_selected_mic,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ClipFlow");
}
