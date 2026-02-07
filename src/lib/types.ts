export interface Region {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Clip {
  id: string;
  path: string;
  duration_ms: number;
  region: Region;
  has_audio: boolean;
  thumbnail_path: string | null;
  trim_start_ms: number;
  trim_end_ms: number;
  audio_paths: string[];
}

export type TransitionType =
  | "fade"
  | "fadeblack"
  | "fadewhite"
  | "dissolve"
  | "zoom"
  | "slide"
  | "slideright"
  | "slideup"
  | "slidedown"
  | "wipeleft"
  | "wiperight"
  | "wipeup"
  | "wipedown"
  | "pixelize"
  | "circleopen"
  | "circleclose"
  | "radial"
  | "smoothleft"
  | "smoothright"
  | "cut";

export interface Transition {
  transition_type: TransitionType;
}

export type RecordingState = "idle" | "recording" | "paused";

export type ExportFormat = "mp4" | "gif";
export type ExportQuality = "high" | "medium" | "low";

export type AudioSource = "none" | "system" | "microphone" | "both";

export interface AudioDevice {
  name: string;
  is_input: boolean;
  is_default: boolean;
}

export interface WindowInfo {
  title: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Annotation {
  id: string;
  kind: AnnotationKind;
  x: number;
  y: number;
  width: number;
  height: number;
  color: string;
  stroke_width: number;
  text: string | null;
  points: [number, number][] | null;
  start_ms: number;
  end_ms: number;
}

export type AnnotationKind = "arrow" | "rectangle" | "circle" | "text" | "freehand";

export interface Subtitle {
  id: string;
  text: string;
  start_ms: number;
  end_ms: number;
  position: SubtitlePosition;
  font_size: number;
  color: string;
}

export type SubtitlePosition = "top" | "center" | "bottom";

export interface ProjectSummary {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  clip_count: number;
  total_duration_ms: number;
}
