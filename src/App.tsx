import { useEffect, useRef, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { useAppStore } from "./stores/appStore";
import { StatusIndicator } from "./components/controls/StatusIndicator";
import { RecordButton } from "./components/controls/RecordButton";
import { ThemeToggle } from "./components/controls/ThemeToggle";
import { AudioSourceSelector } from "./components/controls/AudioSourceSelector";
import { CountdownSelector } from "./components/controls/CountdownSelector";
import { ExportButton } from "./components/export/ExportButton";
import { Timeline } from "./components/timeline/Timeline";
import { VideoPreview } from "./components/preview/VideoPreview";
import { SaveProjectButton } from "./components/project/SaveProjectButton";
import { ProjectListModal } from "./components/project/ProjectListModal";
import { SubtitleEditor } from "./components/subtitles/SubtitleEditor";
import { OnboardingOverlay } from "./components/onboarding/OnboardingOverlay";
import { Logo } from "./components/Logo";
import type { Region } from "./lib/types";
import * as api from "./lib/tauri";

function LiveKeystrokeOverlay() {
  const [keys, setKeys] = useState<{ id: number; key: string; time: number }[]>([]);
  const nextId = useRef(0);

  useEffect(() => {
    const unlisten = listen<string>("keystroke-live", (event) => {
      const id = nextId.current++;
      setKeys((prev) => [...prev.slice(-2), { id, key: event.payload, time: Date.now() }]);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  // Auto-remove keys after 1.8s
  useEffect(() => {
    if (keys.length === 0) return;
    const timer = setInterval(() => {
      const now = Date.now();
      setKeys((prev) => prev.filter((k) => now - k.time < 1800));
    }, 200);
    return () => clearInterval(timer);
  }, [keys.length > 0]);

  if (keys.length === 0) return null;

  return (
    <div className="fixed bottom-6 left-6 z-[100] flex flex-col gap-1.5 pointer-events-none">
      {keys.map((k) => {
        const age = (Date.now() - k.time) / 1800;
        const opacity = Math.max(0, 1 - age);
        return (
          <div
            key={k.id}
            className="px-4 py-2 rounded-full bg-black/55 text-white text-sm font-mono shadow-lg backdrop-blur-sm"
            style={{ opacity, transition: "opacity 0.3s" }}
          >
            {k.key}
          </div>
        );
      })}
    </div>
  );
}

function App() {
  const {
    theme,
    recordingState,
    clips,
    currentRegion,
    audioSource,
    durationMs,
    ffmpegReady,
    ffmpegError,
    exporting,
    exportProgress,
    exportError,
    exportSuccess,
    clearExportError,
    clearExportSuccess,
    previewing,
    previewProgress,
    previewPath,
    previewError,
    previewVideo,
    setPreviewProgress,
    closePreview,
    watermarkEnabled,
    toggleWatermark,
    exportFormat,
    exportQuality,
    setExportFormat,
    setExportQuality,
    refreshState,
    startRecording,
    stopRecording,
    pauseRecording,
    resumeRecording,
    cancelRecording,
    openRegionSelector,
    setCaptureRegion,
    setAudioSource,
    clearRegion,
    updateDuration,
    exportVideo,
    setExportProgress,
    ensureFfmpeg,
    countdownSeconds,
    countdownActive,
    countdownRemaining,
    setCountdownSeconds,
    keystrokeEnabled,
    toggleKeystroke,
    cursorZoomEnabled,
    toggleCursorZoom,
    copyToClipboard,
    saveProject,
    loadProject,
    listProjects,
    deleteProject,
    projects,
    currentProjectId,
    onboardingStep,
    nextOnboardingStep,
    skipOnboarding,
    showOnboarding,
    selectedMic,
    setSelectedMic,
    systemVolume,
    micVolume,
    setSystemVolume,
    setMicVolume,
  } = useAppStore();

  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const countdownRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const [showProjectList, setShowProjectList] = useState(false);
  const [showSubtitles, setShowSubtitles] = useState(false);

  // Init
  useEffect(() => {
    ensureFfmpeg();
    refreshState().catch(console.error);
    listProjects().catch(console.error);
  }, []);

  // Listen for region-selected event from overlay window
  useEffect(() => {
    const unlisten = listen<Region>("region-selected", (event) => {
      setCaptureRegion(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Listen for export progress events
  useEffect(() => {
    const unlisten = listen<number>("export-progress", (event) => {
      setExportProgress(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Listen for preview progress events
  useEffect(() => {
    const unlisten = listen<number>("preview-progress", (event) => {
      setPreviewProgress(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Listen for hotkey-triggered state changes
  useEffect(() => {
    const unlisten = listen<string>("recording-state-changed", () => {
      refreshState().catch(console.error);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Duration timer while recording or paused
  useEffect(() => {
    if (recordingState === "recording" || recordingState === "paused") {
      timerRef.current = setInterval(() => {
        updateDuration();
      }, 200);
    } else {
      if (timerRef.current) {
        clearInterval(timerRef.current);
      }
    }
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, [recordingState]);

  // Countdown timer
  useEffect(() => {
    if (countdownActive && countdownRemaining > 0) {
      countdownRef.current = setInterval(() => {
        const store = useAppStore.getState();
        const remaining = store.countdownRemaining - 1;
        if (remaining <= 0) {
          useAppStore.setState({ countdownActive: false, countdownRemaining: 0 });
          if (countdownRef.current) clearInterval(countdownRef.current);
          // Actually start recording
          api.startRecording().then(() => {
            useAppStore.setState({ recordingState: "recording", durationMs: 0 });
          }).catch(console.error);
        } else {
          useAppStore.setState({ countdownRemaining: remaining });
        }
      }, 1000);
    }
    return () => {
      if (countdownRef.current) clearInterval(countdownRef.current);
    };
  }, [countdownActive]);

  const handleRecord = useCallback(async () => {
    if (recordingState === "idle") {
      await startRecording();
    } else if (recordingState === "recording" || recordingState === "paused") {
      await stopRecording();
    }
  }, [recordingState, startRecording, stopRecording]);

  const handlePause = useCallback(async () => {
    if (recordingState === "recording") {
      await pauseRecording();
    } else if (recordingState === "paused") {
      await resumeRecording();
    }
  }, [recordingState, pauseRecording, resumeRecording]);

  const handleCancel = useCallback(async () => {
    if (recordingState === "recording" || recordingState === "paused") {
      await cancelRecording();
    }
  }, [recordingState, cancelRecording]);

  const handleExport = async () => {
    try {
      await exportVideo();
    } catch (e) {
      console.error("Export failed:", e);
    }
  };

  const handlePreview = async () => {
    try {
      await previewVideo();
    } catch (e) {
      console.error("Preview failed:", e);
    }
  };

  const formatDuration = (ms: number) => {
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    const tenths = Math.floor((ms % 1000) / 100);
    return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}.${tenths}`;
  };

  const extractFilename = (path: string) => {
    const parts = path.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || path;
  };

  const totalDurationMs = clips.reduce((sum, c) => sum + c.duration_ms, 0);

  return (
    <div className={`min-h-screen flex flex-col select-none transition-colors duration-200 ${theme === "dark" ? "dark" : ""}`}>
      <div className="min-h-screen flex flex-col bg-zinc-50 dark:bg-zinc-950 text-zinc-900 dark:text-white">
        {/* Countdown Overlay */}
        {countdownActive && countdownRemaining > 0 && (
          <div className="fixed inset-0 z-[200] flex items-center justify-center bg-black/70">
            <div className="text-center">
              <div className="text-9xl font-bold text-white animate-pulse tabular-nums">
                {countdownRemaining}
              </div>
              <p className="text-zinc-400 text-sm mt-4">L'enregistrement commence dans...</p>
            </div>
          </div>
        )}

        {/* Header */}
        <header className="flex items-center gap-4 px-6 py-4 border-b border-zinc-200 dark:border-zinc-800">
          <div className="flex items-center gap-2.5">
            <Logo size={26} />
            <h1 className="text-lg font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
              ClipFlow
            </h1>
          </div>
          <StatusIndicator state={recordingState} />
          {(recordingState === "recording" || recordingState === "paused") && (
            <span className={`ml-2 font-mono text-sm tabular-nums ${recordingState === "paused" ? "text-yellow-500 dark:text-yellow-400" : "text-red-500 dark:text-red-400"}`}>
              {formatDuration(durationMs)}
            </span>
          )}

          {/* Region indicator */}
          {currentRegion && recordingState === "idle" && !exporting && (
            <div className="ml-auto flex items-center gap-2 text-xs text-zinc-500 dark:text-zinc-400">
              <span className="font-mono">
                {currentRegion.width}x{currentRegion.height}
              </span>
              <span className="text-zinc-400 dark:text-zinc-600">
                ({currentRegion.x}, {currentRegion.y})
              </span>
              <button
                onClick={clearRegion}
                className="text-zinc-400 dark:text-zinc-600 hover:text-zinc-600 dark:hover:text-zinc-400 transition-colors"
                title="Capturer l'écran entier"
              >
                ✕
              </button>
            </div>
          )}

          {!currentRegion && recordingState === "idle" && !exporting && (
            <span className="ml-auto text-xs text-zinc-400 dark:text-zinc-600">Écran entier</span>
          )}

          {!ffmpegReady && !ffmpegError && (
            <span className="text-xs text-yellow-600 dark:text-yellow-500 animate-pulse">
              Téléchargement FFmpeg...
            </span>
          )}
          {ffmpegError && (
            <button
              className="text-xs text-red-500 hover:text-red-400 underline"
              onClick={() => ensureFfmpeg()}
              title={ffmpegError}
            >
              FFmpeg échoué — réessayer
            </button>
          )}

          <ThemeToggle />
        </header>

        {/* Timeline */}
        <main className="flex-1 flex items-center justify-center px-6 py-4" data-onboarding-timeline>
          {clips.length === 0 ? (
            <div className="text-center space-y-3">
              <svg
                className="mx-auto text-zinc-300 dark:text-zinc-700"
                width="48"
                height="48"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18" />
                <line x1="7" y1="2" x2="7" y2="22" />
                <line x1="17" y1="2" x2="17" y2="22" />
                <line x1="2" y1="12" x2="22" y2="12" />
                <line x1="2" y1="7" x2="7" y2="7" />
                <line x1="2" y1="17" x2="7" y2="17" />
                <line x1="17" y1="7" x2="22" y2="7" />
                <line x1="17" y1="17" x2="22" y2="17" />
              </svg>
              <div>
                <p className="text-zinc-400 dark:text-zinc-500 text-sm">Aucun clip enregistré</p>
                <p className="text-zinc-400 dark:text-zinc-600 text-xs mt-1">
                  Sélectionne une zone puis appuie sur <kbd className="px-1.5 py-0.5 bg-zinc-200 dark:bg-zinc-800 rounded text-zinc-600 dark:text-zinc-400 font-mono">F9</kbd> pour enregistrer
                </p>
              </div>
            </div>
          ) : (
            <Timeline />
          )}
        </main>

        {/* Export success */}
        {exportSuccess && (
          <div className="mx-6 mb-2 px-4 py-2.5 bg-emerald-50 dark:bg-emerald-900/40 border border-emerald-200 dark:border-emerald-700/50 border-l-4 border-l-emerald-500 rounded-lg flex items-center gap-3 animate-fade-in">
            <svg className="w-4 h-4 text-emerald-500 dark:text-emerald-400 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="20 6 9 17 4 12" />
            </svg>
            <span className="text-emerald-700 dark:text-emerald-300 text-sm flex-1">
              Export terminé — <span className="font-mono text-emerald-600 dark:text-emerald-400">{extractFilename(exportSuccess)}</span>
            </span>
            <button
              onClick={clearExportSuccess}
              className="text-emerald-400 dark:text-emerald-600 hover:text-emerald-600 dark:hover:text-emerald-400 transition-colors"
            >
              ✕
            </button>
          </div>
        )}

        {/* Export error */}
        {exportError && (
          <div className="mx-6 mb-2 px-4 py-2.5 bg-red-50 dark:bg-red-900/40 border border-red-200 dark:border-red-700/50 rounded-lg flex items-center gap-3 animate-fade-in">
            <svg className="w-4 h-4 text-red-500 dark:text-red-400 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10" />
              <line x1="15" y1="9" x2="9" y2="15" />
              <line x1="9" y1="9" x2="15" y2="15" />
            </svg>
            <span className="text-red-700 dark:text-red-300 text-sm flex-1">{exportError}</span>
            <button
              onClick={clearExportError}
              className="text-red-400 dark:text-red-600 hover:text-red-600 dark:hover:text-red-400 transition-colors"
            >
              ✕
            </button>
          </div>
        )}

        {/* Preview error */}
        {previewError && (
          <div className="mx-6 mb-2 px-4 py-2.5 bg-orange-50 dark:bg-orange-900/40 border border-orange-200 dark:border-orange-700/50 rounded-lg flex items-center gap-3 animate-fade-in">
            <svg className="w-4 h-4 text-orange-500 dark:text-orange-400 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10" />
              <line x1="12" y1="8" x2="12" y2="12" />
              <line x1="12" y1="16" x2="12.01" y2="16" />
            </svg>
            <span className="text-orange-700 dark:text-orange-300 text-sm flex-1">{previewError}</span>
            <button
              onClick={closePreview}
              className="text-orange-400 dark:text-orange-600 hover:text-orange-600 dark:hover:text-orange-400 transition-colors"
            >
              ✕
            </button>
          </div>
        )}

        {/* Controls */}
        <footer className="flex items-center flex-wrap gap-3 gap-y-2 px-6 py-4 border-t border-zinc-200 dark:border-zinc-800">
          <button
            onClick={openRegionSelector}
            disabled={recordingState !== "idle" || exporting}
            className="px-4 py-2.5 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 disabled:opacity-40 disabled:cursor-not-allowed rounded-lg text-sm font-medium transition-colors"
            data-onboarding-region
          >
            Nouvelle Zone
          </button>
          <div data-onboarding-record>
            <RecordButton
              state={recordingState}
              onClick={handleRecord}
              disabled={!ffmpegReady || exporting || countdownActive}
            />
          </div>
          {(recordingState === "recording" || recordingState === "paused") && (
            <>
              <button
                onClick={handlePause}
                className={`px-4 py-2.5 rounded-lg text-sm font-medium transition-colors ${
                  recordingState === "paused"
                    ? "bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-300 hover:bg-green-200 dark:hover:bg-green-900/60"
                    : "bg-yellow-100 dark:bg-yellow-900/40 text-yellow-700 dark:text-yellow-300 hover:bg-yellow-200 dark:hover:bg-yellow-900/60"
                }`}
              >
                {recordingState === "paused" ? "Reprendre" : "Pause"}
              </button>
              <button
                onClick={handleCancel}
                className="px-4 py-2.5 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-lg text-sm font-medium transition-colors text-zinc-600 dark:text-zinc-300"
              >
                Annuler
              </button>
            </>
          )}

          <div className="ml-auto flex items-center flex-wrap gap-3">
            {/* Audio source selector */}
            {recordingState === "idle" && !exporting && (
              <AudioSourceSelector
                audioSource={audioSource}
                onChange={setAudioSource}
                selectedMic={selectedMic}
                onMicChange={setSelectedMic}
                disabled={recordingState !== "idle"}
                systemVolume={systemVolume}
                micVolume={micVolume}
                onSystemVolumeChange={setSystemVolume}
                onMicVolumeChange={setMicVolume}
              />
            )}

            {/* Countdown selector */}
            {recordingState === "idle" && !exporting && (
              <CountdownSelector
                seconds={countdownSeconds}
                onChange={setCountdownSeconds}
                disabled={recordingState !== "idle"}
              />
            )}

            {/* Subtitles button */}
            {clips.length > 0 && recordingState === "idle" && !exporting && (
              <button
                onClick={() => setShowSubtitles(true)}
                className="px-2.5 py-1.5 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-lg text-xs font-medium transition-colors flex items-center gap-1.5"
                title="Sous-titres"
              >
                <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <rect x="2" y="4" width="20" height="16" rx="2" />
                  <line x1="6" y1="14" x2="18" y2="14" />
                  <line x1="6" y1="18" x2="14" y2="18" />
                </svg>
                ST
              </button>
            )}

            {/* Preview button */}
            <button
              onClick={handlePreview}
              disabled={clips.length === 0 || recordingState !== "idle" || exporting || previewing}
              className="px-4 py-2.5 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 disabled:opacity-40 disabled:cursor-not-allowed rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
            >
              {previewing ? (
                <>
                  <svg className="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
                  </svg>
                  <span>{previewProgress}%</span>
                </>
              ) : (
                <>
                  <svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <polygon points="5 3 19 12 5 21 5 3" />
                  </svg>
                  Preview
                </>
              )}
            </button>
            <ExportButton
              clipCount={clips.length}
              exporting={exporting}
              exportProgress={exportProgress}
              exportFormat={exportFormat}
              exportQuality={exportQuality}
              exportSuccess={exportSuccess}
              onExport={handleExport}
              onFormatChange={setExportFormat}
              onQualityChange={setExportQuality}
              onCopyToClipboard={copyToClipboard}
            />
            {!exporting && recordingState === "idle" && (
              <div className="text-[9px] text-zinc-400 dark:text-zinc-600 flex gap-1.5">
                <kbd className="px-1 py-0.5 bg-zinc-100 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded text-zinc-500 font-mono">
                  F9
                </kbd>
                <span>Rec</span>
                <kbd className="px-1 py-0.5 bg-zinc-100 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded text-zinc-500 font-mono">
                  F10
                </kbd>
                <span>Pause</span>
              </div>
            )}
            {!exporting && (recordingState === "recording" || recordingState === "paused") && (
              <div className="text-[9px] text-zinc-400 dark:text-zinc-600 flex gap-1.5">
                <kbd className="px-1 py-0.5 bg-zinc-100 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded text-zinc-500 font-mono">
                  F9
                </kbd>
                <span>Stop</span>
                <kbd className="px-1 py-0.5 bg-zinc-100 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded text-zinc-500 font-mono">
                  F10
                </kbd>
                <span>{recordingState === "paused" ? "Repr." : "Pause"}</span>
                <kbd className="px-1 py-0.5 bg-zinc-100 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded text-zinc-500 font-mono">
                  ESC
                </kbd>
                <span>Annuler</span>
              </div>
            )}
          </div>
        </footer>

        {/* Bottom bar: toggles, project save, credits */}
        <div className="flex items-center justify-between flex-wrap gap-y-2 px-6 py-2 border-t border-zinc-100 dark:border-zinc-900">
          <div className="flex items-center flex-wrap gap-4">
            {/* Watermark toggle */}
            <button
              onClick={toggleWatermark}
              className="flex items-center gap-2 text-xs text-zinc-400 dark:text-zinc-600 hover:text-zinc-600 dark:hover:text-zinc-400 transition-colors"
            >
              <div className={`w-7 h-4 rounded-full transition-colors flex items-center ${watermarkEnabled ? "bg-blue-500 justify-end" : "bg-zinc-300 dark:bg-zinc-700 justify-start"}`}>
                <div className="w-3 h-3 rounded-full bg-white mx-0.5 shadow-sm" />
              </div>
              <span>Filigrane</span>
            </button>

            {/* Keystroke toggle */}
            <button
              onClick={toggleKeystroke}
              className="flex items-center gap-2 text-xs text-zinc-400 dark:text-zinc-600 hover:text-zinc-600 dark:hover:text-zinc-400 transition-colors"
            >
              <div className={`w-7 h-4 rounded-full transition-colors flex items-center ${keystrokeEnabled ? "bg-blue-500 justify-end" : "bg-zinc-300 dark:bg-zinc-700 justify-start"}`}>
                <div className="w-3 h-3 rounded-full bg-white mx-0.5 shadow-sm" />
              </div>
              <span>Touches</span>
            </button>

            {/* Cursor zoom toggle */}
            <button
              onClick={toggleCursorZoom}
              className="flex items-center gap-2 text-xs text-zinc-400 dark:text-zinc-600 hover:text-zinc-600 dark:hover:text-zinc-400 transition-colors"
            >
              <div className={`w-7 h-4 rounded-full transition-colors flex items-center ${cursorZoomEnabled ? "bg-blue-500 justify-end" : "bg-zinc-300 dark:bg-zinc-700 justify-start"}`}>
                <div className="w-3 h-3 rounded-full bg-white mx-0.5 shadow-sm" />
              </div>
              <span>Auto-Zoom</span>
            </button>

            {/* Project save */}
            {recordingState === "idle" && !exporting && (
              <SaveProjectButton
                onSave={saveProject}
                onListOpen={() => { listProjects(); setShowProjectList(true); }}
                hasClips={clips.length > 0}
                disabled={recordingState !== "idle"}
              />
            )}
          </div>

          <div className="flex items-center gap-3">
            <button
              onClick={showOnboarding}
              className="text-[10px] text-zinc-400 dark:text-zinc-600 hover:text-blue-500 transition-colors"
            >
              Tutoriel
            </button>
            <a
              href="https://github.com/thomaslekieffre"
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs text-zinc-400 dark:text-zinc-500 hover:text-blue-500 dark:hover:text-blue-400 transition-colors font-medium"
            >
              Thomas Lekieffre <span className="text-zinc-300 dark:text-zinc-600 font-normal">— DrPepper</span>
            </a>
          </div>
        </div>
      </div>

      {/* Live Keystroke Overlay */}
      {keystrokeEnabled && (recordingState === "recording" || recordingState === "paused") && (
        <LiveKeystrokeOverlay />
      )}

      {/* Video Preview Modal */}
      {previewPath && (
        <VideoPreview
          filePath={previewPath}
          onClose={closePreview}
        />
      )}

      {/* Project List Modal */}
      {showProjectList && (
        <ProjectListModal
          projects={projects}
          currentProjectId={currentProjectId}
          onLoad={(id) => { loadProject(id); setShowProjectList(false); }}
          onDelete={deleteProject}
          onClose={() => setShowProjectList(false)}
        />
      )}

      {/* Subtitle Editor Modal */}
      {showSubtitles && (
        <SubtitleEditor
          totalDurationMs={totalDurationMs}
          onClose={() => setShowSubtitles(false)}
        />
      )}

      {/* Onboarding Overlay */}
      {onboardingStep !== null && (
        <OnboardingOverlay
          step={onboardingStep}
          onNext={nextOnboardingStep}
          onSkip={skipOnboarding}
        />
      )}
    </div>
  );
}

export default App;
