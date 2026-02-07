use crate::types::{AudioSource, Clip, RecordingState, Region, Transition};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Child;
use std::time::Instant;

pub struct AppState {
    pub clips: Vec<Clip>,
    pub transitions: Vec<Transition>,
    pub recording_state: RecordingState,
    pub current_region: Option<Region>,
    pub audio_source: AudioSource,
    pub temp_dir: PathBuf,
    pub ffmpeg_process: Option<Child>,
    pub recording_start: Option<Instant>,
    pub current_clip_path: Option<PathBuf>,
    // Pause support: segmented recording
    pub recording_segments: Vec<PathBuf>,
    pub pause_accumulated_ms: u64,
    pub segment_index: u32,
    // Audio capture handles
    pub audio_handles: Vec<AudioCaptureHandle>,
    pub audio_temp_paths: Vec<PathBuf>,
    // Countdown
    pub countdown_seconds: u32,
    // Keystroke capture
    pub keystroke_enabled: bool,
    pub keystroke_handle: Option<crate::capture::keystroke::KeystrokeCaptureHandle>,
    pub clip_keystrokes: HashMap<String, Vec<crate::types::KeystrokeEvent>>,
    // Cursor tracking
    pub cursor_zoom_enabled: bool,
    pub cursor_handle: Option<crate::capture::cursor::CursorTrackingHandle>,
    pub clip_cursor_positions: HashMap<String, Vec<crate::types::CursorPosition>>,
    // Annotations & Subtitles
    pub annotations: HashMap<String, Vec<crate::types::Annotation>>,
    pub subtitles: Vec<crate::types::Subtitle>,
    // Project
    pub current_project_id: Option<String>,
}

pub struct AudioCaptureHandle {
    pub join_handle: Option<std::thread::JoinHandle<()>>,
    pub stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Default for AppState {
    fn default() -> Self {
        let temp_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ClipFlow")
            .join("temp");

        Self {
            clips: Vec::new(),
            transitions: Vec::new(),
            recording_state: RecordingState::Idle,
            current_region: None,
            audio_source: AudioSource::None,
            temp_dir,
            ffmpeg_process: None,
            recording_start: None,
            current_clip_path: None,
            recording_segments: Vec::new(),
            pause_accumulated_ms: 0,
            segment_index: 0,
            audio_handles: Vec::new(),
            audio_temp_paths: Vec::new(),
            countdown_seconds: 3,
            keystroke_enabled: false,
            keystroke_handle: None,
            clip_keystrokes: HashMap::new(),
            cursor_zoom_enabled: false,
            cursor_handle: None,
            clip_cursor_positions: HashMap::new(),
            annotations: HashMap::new(),
            subtitles: Vec::new(),
            current_project_id: None,
        }
    }
}
