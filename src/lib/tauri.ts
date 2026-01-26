import { invoke } from "@tauri-apps/api/core";
import type {
  Clip,
  RecordingState,
  Region,
  Transition,
  TransitionType,
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

export async function startRecording(): Promise<void> {
  return invoke("start_recording");
}

export async function stopRecording(): Promise<Clip> {
  return invoke("stop_recording");
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

export async function exportVideo(watermark: boolean): Promise<string> {
  return invoke("export_video", { watermark });
}

export async function previewVideo(): Promise<string> {
  return invoke("preview_video");
}

export async function ensureFfmpeg(): Promise<string> {
  return invoke("ensure_ffmpeg");
}
