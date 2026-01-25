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
  | "smoothright";

export interface Transition {
  transition_type: TransitionType;
}

export type RecordingState = "idle" | "recording" | "paused";

export type ExportFormat = "mp4" | "gif";

export interface ExportSettings {
  format: ExportFormat;
  transitions: Transition[];
}
