import { create } from "zustand";
import type {
  AudioSource,
  Clip,
  ExportFormat,
  ExportQuality,
  ProjectSummary,
  RecordingState,
  Region,
  Transition,
  TransitionType,
} from "../lib/types";
import * as api from "../lib/tauri";

type Theme = "light" | "dark";

interface AppStore {
  // State
  theme: Theme;
  recordingState: RecordingState;
  clips: Clip[];
  transitions: Transition[];
  currentRegion: Region | null;
  audioSource: AudioSource;
  durationMs: number;
  ffmpegReady: boolean;
  ffmpegError: string | null;
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
  // Countdown
  countdownSeconds: number;
  countdownActive: boolean;
  countdownRemaining: number;
  // Keystroke display
  keystrokeEnabled: boolean;
  // Cursor zoom
  cursorZoomEnabled: boolean;
  // Projects
  projects: ProjectSummary[];
  currentProjectId: string | null;
  // Mic selection
  selectedMic: string | null;
  // Audio volumes
  systemVolume: number;
  micVolume: number;
  // Onboarding
  onboardingStep: number | null;

  // Actions
  refreshState: () => Promise<void>;
  startRecording: () => Promise<void>;
  stopRecording: () => Promise<Clip>;
  pauseRecording: () => Promise<void>;
  resumeRecording: () => Promise<void>;
  cancelRecording: () => Promise<void>;
  setAudioSource: (source: AudioSource) => Promise<void>;
  deleteClip: (clipId: string) => Promise<void>;
  setTransition: (index: number, type_: TransitionType, durationS?: number) => Promise<void>;
  setAllTransitions: (type_: TransitionType) => Promise<void>;
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
  // Countdown
  setCountdownSeconds: (seconds: number) => void;
  startCountdown: () => void;
  // Keystroke
  toggleKeystroke: () => Promise<void>;
  // Cursor zoom
  toggleCursorZoom: () => Promise<void>;
  // Clipboard export
  copyToClipboard: (path: string) => Promise<void>;
  // Projects
  saveProject: (name: string) => Promise<void>;
  loadProject: (projectId: string) => Promise<void>;
  listProjects: () => Promise<void>;
  deleteProject: (projectId: string) => Promise<void>;
  // Mic selection
  setSelectedMic: (deviceName: string | null) => Promise<void>;
  // Audio volumes
  setSystemVolume: (volume: number) => Promise<void>;
  setMicVolume: (volume: number) => Promise<void>;
  // Onboarding
  showOnboarding: () => void;
  nextOnboardingStep: () => void;
  skipOnboarding: () => void;
  // Transition presets
  applyTransitionPreset: (preset: string) => Promise<void>;
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
  audioSource: "none",
  durationMs: 0,
  ffmpegReady: false,
  ffmpegError: null,
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
  countdownSeconds: (() => {
    try {
      const saved = localStorage.getItem("clipflow-countdown");
      if (saved) {
        const n = parseInt(saved, 10);
        if (n === 0 || n === 3 || n === 5) return n;
      }
    } catch {}
    return 3;
  })(),
  countdownActive: false,
  countdownRemaining: 0,
  keystrokeEnabled: false,
  cursorZoomEnabled: false,
  selectedMic: null,
  systemVolume: (() => {
    try {
      const saved = localStorage.getItem("clipflow-system-volume");
      if (saved) { const n = parseFloat(saved); if (!isNaN(n)) return n; }
    } catch {}
    return 1.0;
  })(),
  micVolume: (() => {
    try {
      const saved = localStorage.getItem("clipflow-mic-volume");
      if (saved) { const n = parseFloat(saved); if (!isNaN(n)) return n; }
    } catch {}
    return 1.0;
  })(),
  projects: [],
  currentProjectId: null,
  onboardingStep: (() => {
    try {
      const seen = localStorage.getItem("clipflow-onboarding-done");
      return seen ? null : 0;
    } catch { return 0; }
  })(),

  refreshState: async () => {
    const [recordingState, clips, transitions] = await Promise.all([
      api.getRecordingState(),
      api.getClips(),
      api.getTransitions(),
    ]);
    set({ recordingState, clips, transitions });
    // Sync audio volumes from local storage to backend
    const { systemVolume, micVolume } = get();
    api.setAudioVolumes(systemVolume, micVolume).catch(() => {});
  },

  startRecording: async () => {
    const { countdownSeconds } = get();
    if (countdownSeconds > 0) {
      set({ countdownActive: true, countdownRemaining: countdownSeconds });
      // Countdown is handled in App.tsx via interval
      return;
    }
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

  pauseRecording: async () => {
    await api.pauseRecording();
    set({ recordingState: "paused" });
  },

  resumeRecording: async () => {
    await api.resumeRecording();
    set({ recordingState: "recording" });
  },

  cancelRecording: async () => {
    await api.cancelRecording();
    set({ recordingState: "idle", durationMs: 0 });
  },

  setAudioSource: async (source: AudioSource) => {
    await api.setAudioSource(source);
    localStorage.setItem("clipflow-audio-source", source);
    set({ audioSource: source });
  },

  deleteClip: async (clipId: string) => {
    await api.deleteClip(clipId);
    const [clips, transitions] = await Promise.all([
      api.getClips(),
      api.getTransitions(),
    ]);
    set({ clips, transitions });
  },

  setTransition: async (index: number, type_: TransitionType, durationS?: number) => {
    await api.setTransition(index, type_, durationS);
    const transitions = await api.getTransitions();
    set({ transitions });
  },

  setAllTransitions: async (type_: TransitionType) => {
    await api.setAllTransitions(type_);
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
    const state = get().recordingState;
    if (state === "recording" || state === "paused") {
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
    try {
      set({ ffmpegError: null });
      await api.ensureFfmpeg();
      set({ ffmpegReady: true });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error("FFmpeg init failed:", msg);
      set({ ffmpegError: msg });
    }
  },

  setCountdownSeconds: (seconds: number) => {
    localStorage.setItem("clipflow-countdown", String(seconds));
    set({ countdownSeconds: seconds });
  },

  startCountdown: () => {
    set({ countdownActive: true, countdownRemaining: get().countdownSeconds });
  },

  toggleKeystroke: async () => {
    const enabled = await api.toggleKeystrokeDisplay();
    set({ keystrokeEnabled: enabled });
  },

  toggleCursorZoom: async () => {
    const enabled = await api.toggleCursorZoom();
    set({ cursorZoomEnabled: enabled });
  },

  copyToClipboard: async (path: string) => {
    await api.copyFileToClipboard(path);
  },

  saveProject: async (name: string) => {
    const id = await api.saveProject(name);
    set({ currentProjectId: id });
    await get().listProjects();
  },

  loadProject: async (projectId: string) => {
    await api.loadProject(projectId);
    set({ currentProjectId: projectId });
    await get().refreshState();
  },

  listProjects: async () => {
    const projects = await api.listProjects();
    set({ projects });
  },

  deleteProject: async (projectId: string) => {
    await api.deleteProject(projectId);
    if (get().currentProjectId === projectId) {
      set({ currentProjectId: null });
    }
    await get().listProjects();
  },

  setSelectedMic: async (deviceName: string | null) => {
    await api.setSelectedMic(deviceName);
    set({ selectedMic: deviceName });
  },

  setSystemVolume: async (volume: number) => {
    const clamped = Math.max(0, Math.min(2, volume));
    localStorage.setItem("clipflow-system-volume", String(clamped));
    set({ systemVolume: clamped });
    await api.setAudioVolumes(clamped, get().micVolume);
  },

  setMicVolume: async (volume: number) => {
    const clamped = Math.max(0, Math.min(2, volume));
    localStorage.setItem("clipflow-mic-volume", String(clamped));
    set({ micVolume: clamped });
    await api.setAudioVolumes(get().systemVolume, clamped);
  },

  showOnboarding: () => {
    set({ onboardingStep: 0 });
  },

  nextOnboardingStep: () => {
    const step = get().onboardingStep;
    if (step !== null && step < 4) {
      set({ onboardingStep: step + 1 });
    } else {
      localStorage.setItem("clipflow-onboarding-done", "true");
      set({ onboardingStep: null });
    }
  },

  skipOnboarding: () => {
    localStorage.setItem("clipflow-onboarding-done", "true");
    set({ onboardingStep: null });
  },

  applyTransitionPreset: async (preset: string) => {
    const transitions = get().transitions;
    if (transitions.length === 0) return;

    switch (preset) {
      case "professional":
        await api.setAllTransitions("fade");
        break;
      case "dynamic": {
        const dynamicTypes: TransitionType[] = ["slide", "zoom", "slideright", "slideup"];
        for (let i = 0; i < transitions.length; i++) {
          await api.setTransition(i, dynamicTypes[i % dynamicTypes.length]);
        }
        break;
      }
      case "minimal":
        await api.setAllTransitions("cut");
        break;
      case "creative": {
        const creativeTypes: TransitionType[] = ["circleopen", "pixelize", "radial", "dissolve", "wipeleft"];
        for (let i = 0; i < transitions.length; i++) {
          await api.setTransition(i, creativeTypes[i % creativeTypes.length]);
        }
        break;
      }
    }

    const updated = await api.getTransitions();
    set({ transitions: updated });
  },
}));
