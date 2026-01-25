use crate::types::{Clip, RecordingState, Region, Transition};
use std::path::PathBuf;
use tokio::process::Child;
use std::time::Instant;

pub struct AppState {
    pub clips: Vec<Clip>,
    pub transitions: Vec<Transition>,
    pub recording_state: RecordingState,
    pub current_region: Option<Region>,
    pub audio_enabled: bool,
    pub temp_dir: PathBuf,
    pub ffmpeg_process: Option<Child>,
    pub recording_start: Option<Instant>,
    pub current_clip_path: Option<PathBuf>,
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
            audio_enabled: false,
            temp_dir,
            ffmpeg_process: None,
            recording_start: None,
            current_clip_path: None,
        }
    }
}
