import { create } from "zustand";
import type { Clip, ExportFormat, ExportQuality, RecordingState, Region, Transition, TransitionType } from "../lib/types";
import * as api from "../lib/tauri";

type Theme = "light" | "dark";

interface AppStore {
  // State
  theme: Theme;
  recordingState: RecordingState;
  clips: Clip[];
  transitions: Transition[];
  currentRegion: Region | null;
  audioEnabled: boolean;
  durationMs: number;
  ffmpegReady: boolean;
  exporting: boolean;
  exportProgress: number;
  exportError: string | null;
  exportSuccess: string | null;
  previewing: boolean;
  previewProgress: number;
  previewPath: string | null;
  previewError: string | null;
  watermarkEnabled: boolean;
  exportFormat: ExportFormat;
  exportQuality: ExportQuality;

  // Actions
  refreshState: () => Promise<void>;
  startRecording: () => Promise<void>;
  stopRecording: () => Promise<Clip>;
  cancelRecording: () => Promise<void>;
  toggleAudio: () => Promise<void>;
  deleteClip: (clipId: string) => Promise<void>;
  setTransition: (index: number, type_: TransitionType) => Promise<void>;
  reorderClips: (clipIds: string[]) => Promise<void>;
  setCaptureRegion: (region: Region) => Promise<void>;
  openRegionSelector: () => Promise<void>;
  clearRegion: () => void;
  updateDuration: () => Promise<void>;
  exportVideo: () => Promise<string>;
  setExportProgress: (progress: number) => void;
  clearExportError: () => void;
  clearExportSuccess: () => void;
  previewVideo: () => Promise<void>;
  setPreviewProgress: (progress: number) => void;
  closePreview: () => void;
  toggleWatermark: () => void;
  setExportFormat: (format: ExportFormat) => void;
  setExportQuality: (quality: ExportQuality) => void;
  setClipTrim: (clipId: string, trimStartMs: number, trimEndMs: number) => Promise<void>;
  toggleTheme: () => void;
  ensureFfmpeg: () => Promise<void>;
}

const getInitialTheme = (): Theme => {
  try {
    const saved = localStorage.getItem("clipflow-theme");
    if (saved === "light" || saved === "dark") return saved;
  } catch {}
  return "dark";
};

export const useAppStore = create<AppStore>((set, get) => ({
  theme: getInitialTheme(),
  recordingState: "idle",
  clips: [],
  transitions: [],
  currentRegion: null,
  audioEnabled: false,
  durationMs: 0,
  ffmpegReady: false,
  exporting: false,
  exportProgress: 0,
  exportError: null,
  exportSuccess: null,
  previewing: false,
  previewProgress: 0,
  previewPath: null,
  previewError: null,
  exportFormat: ((): ExportFormat => {
    try {
      const saved = localStorage.getItem("clipflow-format");
      if (saved === "mp4" || saved === "gif") return saved;
    } catch {}
    return "mp4";
  })(),
  exportQuality: ((): ExportQuality => {
    try {
      const saved = localStorage.getItem("clipflow-quality");
      if (saved === "high" || saved === "medium" || saved === "low") return saved;
    } catch {}
    return "medium";
  })(),
  watermarkEnabled: (() => {
    try {
      const saved = localStorage.getItem("clipflow-watermark");
      return saved === null ? true : saved === "true";
    } catch { return true; }
  })(),

  refreshState: async () => {
    const [recordingState, clips, transitions] = await Promise.all([
      api.getRecordingState(),
      api.getClips(),
      api.getTransitions(),
    ]);
    set({ recordingState, clips, transitions });
  },

  startRecording: async () => {
    await api.startRecording();
    set({ recordingState: "recording", durationMs: 0 });
  },

  stopRecording: async () => {
    const clip = await api.stopRecording();
    const [clips, transitions] = await Promise.all([
      api.getClips(),
      api.getTransitions(),
    ]);
    set({ recordingState: "idle", clips, transitions, durationMs: 0 });
    return clip;
  },

  cancelRecording: async () => {
    await api.cancelRecording();
    set({ recordingState: "idle", durationMs: 0 });
  },

  toggleAudio: async () => {
    const enabled = await api.toggleAudio();
    set({ audioEnabled: enabled });
  },

  deleteClip: async (clipId: string) => {
    await api.deleteClip(clipId);
    const [clips, transitions] = await Promise.all([
      api.getClips(),
      api.getTransitions(),
    ]);
    set({ clips, transitions });
  },

  setTransition: async (index: number, type_: TransitionType) => {
    await api.setTransition(index, type_);
    const transitions = await api.getTransitions();
    set({ transitions });
  },

  reorderClips: async (clipIds: string[]) => {
    await api.reorderClips(clipIds);
    const [clips, transitions] = await Promise.all([
      api.getClips(),
      api.getTransitions(),
    ]);
    set({ clips, transitions });
  },

  setCaptureRegion: async (region: Region) => {
    await api.setCaptureRegion(region);
    set({ currentRegion: region });
  },

  openRegionSelector: async () => {
    await api.openRegionSelector();
  },

  clearRegion: () => {
    set({ currentRegion: null });
  },

  updateDuration: async () => {
    if (get().recordingState === "recording") {
      const ms = await api.getRecordingDurationMs();
      set({ durationMs: ms });
    }
  },

  exportVideo: async () => {
    set({ exporting: true, exportProgress: 0, exportError: null, exportSuccess: null });
    try {
      const path = await api.exportVideo(get().watermarkEnabled, get().exportFormat, get().exportQuality);
      set({ exporting: false, exportProgress: 100, exportSuccess: path });
      return path;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      set({ exporting: false, exportProgress: 0, exportError: msg });
      throw e;
    }
  },

  setExportProgress: (progress: number) => {
    set({ exportProgress: progress });
  },

  clearExportError: () => {
    set({ exportError: null });
  },

  clearExportSuccess: () => {
    set({ exportSuccess: null });
  },

  previewVideo: async () => {
    set({ previewing: true, previewProgress: 0, previewPath: null, previewError: null });
    try {
      const path = await api.previewVideo();
      set({ previewing: false, previewProgress: 100, previewPath: path });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      set({ previewing: false, previewProgress: 0, previewError: msg });
    }
  },

  setPreviewProgress: (progress: number) => {
    set({ previewProgress: progress });
  },

  closePreview: () => {
    set({ previewPath: null, previewError: null });
  },

  toggleWatermark: () => {
    const next = !get().watermarkEnabled;
    localStorage.setItem("clipflow-watermark", String(next));
    set({ watermarkEnabled: next });
  },

  setExportFormat: (format: ExportFormat) => {
    localStorage.setItem("clipflow-format", format);
    set({ exportFormat: format });
  },

  setExportQuality: (quality: ExportQuality) => {
    localStorage.setItem("clipflow-quality", quality);
    set({ exportQuality: quality });
  },

  setClipTrim: async (clipId: string, trimStartMs: number, trimEndMs: number) => {
    await api.setClipTrim(clipId, trimStartMs, trimEndMs);
    const clips = await api.getClips();
    set({ clips });
  },

  toggleTheme: () => {
    const next = get().theme === "dark" ? "light" : "dark";
    localStorage.setItem("clipflow-theme", next);
    set({ theme: next });
  },

  ensureFfmpeg: async () => {
    await api.ensureFfmpeg();
    set({ ffmpegReady: true });
  },
}));
