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
    #[serde(default)]
    pub trim_start_ms: u64,
    #[serde(default)]
    pub trim_end_ms: u64,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportQuality {
    High,
    Medium,
    Low,
}

impl ExportQuality {
    pub fn crf(&self) -> u32 {
        match self {
            ExportQuality::High => 18,
            ExportQuality::Medium => 23,
            ExportQuality::Low => 28,
        }
    }

    pub fn preset(&self) -> &'static str {
        match self {
            ExportQuality::High => "slow",
            ExportQuality::Medium => "medium",
            ExportQuality::Low => "fast",
        }
    }
}
