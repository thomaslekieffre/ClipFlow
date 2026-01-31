use crate::recording::manager;
use crate::state::AppState;
use crate::types::RecordingState;
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, Shortcut, ShortcutState};

pub fn handler(app: &tauri::AppHandle, shortcut: &Shortcut, event: tauri_plugin_global_shortcut::ShortcutEvent) {
    if event.state != ShortcutState::Pressed {
        return;
    }

    match shortcut.key {
        Code::F9 => handle_f9(app),
        Code::Escape => handle_escape(app),
        _ => {}
    }
}

fn handle_f9(app: &tauri::AppHandle) {
    let state = app.state::<Mutex<AppState>>();
    let current_state = {
        let Ok(s) = state.lock() else {
            eprintln!("[hotkey] Failed to lock state");
            return;
        };
        s.recording_state
    };

    match current_state {
        RecordingState::Idle => {
            if let Err(e) = manager::start(&state) {
                eprintln!("[hotkey] Start recording failed: {}", e);
            } else {
                eprintln!("[hotkey] Recording started via F9");
                // Notify frontend of state change
                let _ = app.emit("recording-state-changed", "recording");
            }
        }
        RecordingState::Recording => {
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                let state = app_clone.state::<Mutex<AppState>>();
                match manager::stop(&state).await {
                    Ok(_clip) => {
                        eprintln!("[hotkey] Recording stopped via F9");
                        let _ = app_clone.emit("recording-state-changed", "idle");
                    }
                    Err(e) => {
                        eprintln!("[hotkey] Stop recording failed: {}", e);
                    }
                }
            });
        }
        _ => {}
    }
}

fn handle_escape(app: &tauri::AppHandle) {
    let state = app.state::<Mutex<AppState>>();
    let current_state = {
        let Ok(s) = state.lock() else {
            eprintln!("[hotkey] Failed to lock state");
            return;
        };
        s.recording_state
    };

    if current_state == RecordingState::Recording {
        if let Err(e) = manager::cancel(&state) {
            eprintln!("[hotkey] Cancel recording failed: {}", e);
        } else {
            eprintln!("[hotkey] Recording cancelled via ESC");
            let _ = app.emit("recording-state-changed", "idle");
        }
    }
}
