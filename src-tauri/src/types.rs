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
    #[serde(default)]
    pub audio_paths: Vec<String>,
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
    Cut,
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

// Audio source selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioSource {
    None,
    System,
    Microphone,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub is_input: bool,
    pub is_default: bool,
}

// Window info for snapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

// Annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub kind: AnnotationKind,
    pub x: f64,       // normalized 0-1
    pub y: f64,       // normalized 0-1
    pub width: f64,   // normalized 0-1
    pub height: f64,  // normalized 0-1
    pub color: String,
    pub stroke_width: f64,
    pub text: Option<String>,
    pub points: Option<Vec<(f64, f64)>>, // for freehand
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationKind {
    Arrow,
    Rectangle,
    Circle,
    Text,
    Freehand,
}

// Subtitles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtitle {
    pub id: String,
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub position: SubtitlePosition,
    pub font_size: u32,
    pub color: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubtitlePosition {
    Top,
    Center,
    Bottom,
}

// Keystroke events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeEvent {
    pub timestamp_ms: u64,
    pub key_name: String,
}

// Cursor position for auto-zoom
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub timestamp_ms: u64,
    pub x: f64, // relative to region 0-1
    pub y: f64, // relative to region 0-1
}

// Project types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub clips: Vec<Clip>,
    pub transitions: Vec<Transition>,
    pub settings: ProjectSettings,
    pub annotations: std::collections::HashMap<String, Vec<Annotation>>,
    pub subtitles: Vec<Subtitle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub audio_source: AudioSource,
    pub watermark_enabled: bool,
    pub export_format: ExportFormat,
    pub export_quality: ExportQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub clip_count: usize,
    pub total_duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_quality_crf() {
        assert_eq!(ExportQuality::High.crf(), 18);
        assert_eq!(ExportQuality::Medium.crf(), 23);
        assert_eq!(ExportQuality::Low.crf(), 28);
    }

    #[test]
    fn test_export_quality_preset() {
        assert_eq!(ExportQuality::High.preset(), "slow");
        assert_eq!(ExportQuality::Medium.preset(), "medium");
        assert_eq!(ExportQuality::Low.preset(), "fast");
    }

    #[test]
    fn test_transition_default() {
        let t = Transition::default();
        assert_eq!(t.transition_type, TransitionType::Fade);
    }

    #[test]
    fn test_recording_state_initial() {
        let state = RecordingState::Idle;
        assert_eq!(state, RecordingState::Idle);
    }

    #[test]
    fn test_transition_types_equality() {
        assert_eq!(TransitionType::Fade, TransitionType::Fade);
        assert_ne!(TransitionType::Fade, TransitionType::Cut);
    }

    #[test]
    fn test_audio_source_equality() {
        assert_eq!(AudioSource::None, AudioSource::None);
        assert_ne!(AudioSource::System, AudioSource::Microphone);
    }
}
