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
            let img = image::load_from_memory(png_bytes).expect("Failed to decode icon");
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let icon = tauri::image::Image::new_owned(rgba.into_raw(), w, h);
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.set_icon(icon);
            }
            Ok(())
        })
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts(["F9", "Escape"])
                .expect("Failed to register shortcuts")
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
