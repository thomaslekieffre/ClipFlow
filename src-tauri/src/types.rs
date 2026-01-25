use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub path: PathBuf,
    pub duration_ms: u64,
    pub region: Region,
    pub has_audio: bool,
    pub thumbnail_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransitionType {
    Fade,
    FadeBlack,
    FadeWhite,
    Dissolve,
    Zoom,
    Slide,
    SlideRight,
    SlideUp,
    SlideDown,
    WipeLeft,
    WipeRight,
    WipeUp,
    WipeDown,
    Pixelize,
    CircleOpen,
    CircleClose,
    Radial,
    SmoothLeft,
    SmoothRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub transition_type: TransitionType,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            transition_type: TransitionType::Fade,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingState {
    Idle,
    Recording,
    Paused,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Mp4,
    Gif,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    pub format: ExportFormat,
    pub transitions: Vec<Transition>,
}
