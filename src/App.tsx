import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { useAppStore } from "./stores/appStore";
import { StatusIndicator } from "./components/controls/StatusIndicator";
import { RecordButton } from "./components/controls/RecordButton";
import { ThemeToggle } from "./components/controls/ThemeToggle";
import { ExportButton } from "./components/export/ExportButton";
import { Timeline } from "./components/timeline/Timeline";
import { VideoPreview } from "./components/preview/VideoPreview";
import { Logo } from "./components/Logo";
import type { Region } from "./lib/types";

function App() {
  const {
    theme,
    recordingState,
    clips,
    currentRegion,
    durationMs,
    ffmpegReady,
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
    refreshState,
    startRecording,
    stopRecording,
    cancelRecording,
    openRegionSelector,
    setCaptureRegion,
    clearRegion,
    updateDuration,
    exportVideo,
    setExportProgress,
    ensureFfmpeg,
  } = useAppStore();

  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Init
  useEffect(() => {
    ensureFfmpeg().catch(console.error);
    refreshState().catch(console.error);
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

  // Duration timer while recording
  useEffect(() => {
    if (recordingState === "recording") {
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

  // Auto-dismiss export success after 5s
  useEffect(() => {
    if (exportSuccess) {
      const timer = setTimeout(clearExportSuccess, 5000);
      return () => clearTimeout(timer);
    }
  }, [exportSuccess]);

  const handleRecord = async () => {
    if (recordingState === "idle") {
      await startRecording();
    } else if (recordingState === "recording") {
      await stopRecording();
    }
  };

  const handleCancel = async () => {
    if (recordingState === "recording") {
      await cancelRecording();
    }
  };

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

  return (
    <div className={`min-h-screen flex flex-col select-none transition-colors duration-200 ${theme === "dark" ? "dark" : ""}`}>
      <div className="min-h-screen flex flex-col bg-zinc-50 dark:bg-zinc-950 text-zinc-900 dark:text-white">
        {/* Header */}
        <header className="flex items-center gap-4 px-6 py-4 border-b border-zinc-200 dark:border-zinc-800">
          <div className="flex items-center gap-2.5">
            <Logo size={26} />
            <h1 className="text-lg font-bold tracking-tight text-zinc-900 dark:text-zinc-100">
              ClipFlow
            </h1>
          </div>
          <StatusIndicator state={recordingState} />
          {recordingState === "recording" && (
            <span className="ml-2 font-mono text-sm text-red-500 dark:text-red-400 tabular-nums">
              {formatDuration(durationMs)}
            </span>
          )}

          {/* Region indicator */}
          {currentRegion && recordingState === "idle" && !exporting && (
            <div className="ml-auto flex items-center gap-2 text-xs text-zinc-500 dark:text-zinc-400">
              <span className="font-mono">
                {currentRegion.width}×{currentRegion.height}
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

          {!ffmpegReady && (
            <span className="text-xs text-yellow-600 dark:text-yellow-500 animate-pulse">
              Téléchargement FFmpeg...
            </span>
          )}

          <ThemeToggle />
        </header>

        {/* Timeline */}
        <main className="flex-1 flex items-center justify-center px-6 py-4">
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
          <div className="mx-6 mb-2 px-4 py-2.5 bg-emerald-50 dark:bg-emerald-900/40 border border-emerald-200 dark:border-emerald-700/50 rounded-lg flex items-center gap-3 animate-fade-in">
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
        <footer className="flex items-center gap-3 px-6 py-4 border-t border-zinc-200 dark:border-zinc-800">
          <button
            onClick={openRegionSelector}
            disabled={recordingState !== "idle" || exporting}
            className="px-4 py-2.5 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 disabled:opacity-40 disabled:cursor-not-allowed rounded-lg text-sm font-medium transition-colors"
          >
            Nouvelle Zone
          </button>
          <RecordButton
            state={recordingState}
            onClick={handleRecord}
            disabled={!ffmpegReady || exporting}
          />
          {recordingState === "recording" && (
            <button
              onClick={handleCancel}
              className="px-4 py-2.5 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-lg text-sm font-medium transition-colors text-zinc-600 dark:text-zinc-300"
            >
              Annuler
            </button>
          )}

          <div className="ml-auto flex items-center gap-3">
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
              onExport={handleExport}
            />
            {!exporting && (
              <div className="text-xs text-zinc-400 dark:text-zinc-600 flex gap-3">
                <kbd className="px-1.5 py-0.5 bg-zinc-100 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded text-zinc-500 font-mono">
                  F9
                </kbd>
                <span>{recordingState === "recording" ? "Stop" : "Record"}</span>
                {recordingState === "recording" && (
                  <>
                    <kbd className="px-1.5 py-0.5 bg-zinc-100 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded text-zinc-500 font-mono">
                      ESC
                    </kbd>
                    <span>Annuler</span>
                  </>
                )}
              </div>
            )}
          </div>
        </footer>

        {/* Watermark toggle + Credits */}
        <div className="flex items-center justify-between px-6 py-2 border-t border-zinc-100 dark:border-zinc-900">
          <button
            onClick={toggleWatermark}
            className="flex items-center gap-2 text-xs text-zinc-400 dark:text-zinc-600 hover:text-zinc-600 dark:hover:text-zinc-400 transition-colors"
          >
            <div className={`w-7 h-4 rounded-full transition-colors flex items-center ${watermarkEnabled ? "bg-blue-500 justify-end" : "bg-zinc-300 dark:bg-zinc-700 justify-start"}`}>
              <div className="w-3 h-3 rounded-full bg-white mx-0.5 shadow-sm" />
            </div>
            <span>Filigrane</span>
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

      {/* Video Preview Modal */}
      {previewPath && (
        <VideoPreview
          filePath={previewPath}
          onClose={closePreview}
        />
      )}
    </div>
  );
}

export default App;
