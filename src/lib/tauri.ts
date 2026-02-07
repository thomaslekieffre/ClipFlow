import { invoke } from "@tauri-apps/api/core";
import type {
  Annotation,
  AudioDevice,
  AudioSource,
  Clip,
  ExportFormat,
  ExportQuality,
  ProjectSummary,
  RecordingState,
  Region,
  Subtitle,
  Transition,
  TransitionType,
  WindowInfo,
} from "./types";

export async function getRecordingState(): Promise<RecordingState> {
  return invoke("get_recording_state");
}

export async function setCaptureRegion(region: Region): Promise<void> {
  return invoke("set_capture_region", { region });
}

export async function toggleAudio(): Promise<boolean> {
  return invoke("toggle_audio");
}

export async function setAudioSource(source: AudioSource): Promise<void> {
  return invoke("set_audio_source", { source });
}

export async function getAudioSource(): Promise<AudioSource> {
  return invoke("get_audio_source");
}

export async function getAudioDevices(): Promise<AudioDevice[]> {
  return invoke("get_audio_devices");
}

export async function getClips(): Promise<Clip[]> {
  return invoke("get_clips");
}

export async function reorderClips(clipIds: string[]): Promise<void> {
  return invoke("reorder_clips", { clipIds });
}

export async function deleteClip(clipId: string): Promise<void> {
  return invoke("delete_clip", { clipId });
}

export async function setTransition(
  index: number,
  transitionType: TransitionType,
): Promise<void> {
  return invoke("set_transition", { index, transitionType });
}

export async function setAllTransitions(transitionType: TransitionType): Promise<void> {
  return invoke("set_all_transitions", { transitionType });
}

export async function startRecording(): Promise<void> {
  return invoke("start_recording");
}

export async function stopRecording(): Promise<Clip> {
  return invoke("stop_recording");
}

export async function pauseRecording(): Promise<void> {
  return invoke("pause_recording");
}

export async function resumeRecording(): Promise<void> {
  return invoke("resume_recording");
}

export async function cancelRecording(): Promise<void> {
  return invoke("cancel_recording");
}

export async function getRecordingDurationMs(): Promise<number> {
  return invoke("get_recording_duration_ms");
}

export async function openRegionSelector(): Promise<void> {
  return invoke("open_region_selector");
}

export async function closeRegionSelector(): Promise<void> {
  return invoke("close_region_selector");
}

export async function getThumbnailBase64(clipId: string): Promise<string | null> {
  return invoke("get_thumbnail_base64", { clipId });
}

export async function getTransitions(): Promise<Transition[]> {
  return invoke("get_transitions");
}

export async function exportVideo(watermark: boolean, format: ExportFormat, quality: ExportQuality): Promise<string> {
  return invoke("export_video", { watermark, format, quality });
}

export async function setClipTrim(clipId: string, trimStartMs: number, trimEndMs: number): Promise<void> {
  return invoke("set_clip_trim", { clipId, trimStartMs, trimEndMs });
}

export async function previewVideo(): Promise<string> {
  return invoke("preview_video");
}

export async function ensureFfmpeg(): Promise<string> {
  return invoke("ensure_ffmpeg");
}

export async function getVisibleWindows(): Promise<WindowInfo[]> {
  return invoke("get_visible_windows");
}

export async function getMonitorsInfo(): Promise<Region[]> {
  return invoke("get_monitors_info");
}

export async function setCountdown(seconds: number): Promise<void> {
  return invoke("set_countdown", { seconds });
}

export async function getCountdown(): Promise<number> {
  return invoke("get_countdown");
}

export async function setClipAnnotations(clipId: string, annotations: Annotation[]): Promise<void> {
  return invoke("set_clip_annotations", { clipId, annotations });
}

export async function getClipAnnotations(clipId: string): Promise<Annotation[]> {
  return invoke("get_clip_annotations", { clipId });
}

export async function setSubtitles(subtitles: Subtitle[]): Promise<void> {
  return invoke("set_subtitles", { subtitles });
}

export async function getSubtitles(): Promise<Subtitle[]> {
  return invoke("get_subtitles");
}

export async function toggleKeystrokeDisplay(): Promise<boolean> {
  return invoke("toggle_keystroke_display");
}

export async function getKeystrokeEnabled(): Promise<boolean> {
  return invoke("get_keystroke_enabled");
}

export async function toggleCursorZoom(): Promise<boolean> {
  return invoke("toggle_cursor_zoom");
}

export async function getCursorZoomEnabled(): Promise<boolean> {
  return invoke("get_cursor_zoom_enabled");
}

export async function copyFileToClipboard(path: string): Promise<void> {
  return invoke("copy_file_to_clipboard", { path });
}

export async function saveProject(name: string): Promise<string> {
  return invoke("save_project", { name });
}

export async function loadProject(projectId: string): Promise<void> {
  return invoke("load_project", { projectId });
}

export async function listProjects(): Promise<ProjectSummary[]> {
  return invoke("list_projects");
}

export async function deleteProject(projectId: string): Promise<void> {
  return invoke("delete_project", { projectId });
}
