mod capture;
mod commands;
mod export;
mod hotkeys;
mod recording;
mod region;
mod state;
mod types;

use state::AppState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
                .with_shortcuts(["F9", "Escape"])
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
            commands::get_clips,
            commands::reorder_clips,
            commands::delete_clip,
            commands::set_transition,
            commands::start_recording,
            commands::stop_recording,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running ClipFlow");
}
