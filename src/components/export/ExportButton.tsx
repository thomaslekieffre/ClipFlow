import { useState, useEffect } from "react";
import type { ExportFormat, ExportQuality } from "../../lib/types";

interface Props {
  clipCount: number;
  exporting: boolean;
  exportProgress: number;
  exportFormat: ExportFormat;
  exportQuality: ExportQuality;
  exportSuccess: string | null;
  onExport: () => void;
  onFormatChange: (format: ExportFormat) => void;
  onQualityChange: (quality: ExportQuality) => void;
  onCopyToClipboard: (path: string) => void;
}

export function ExportButton({
  clipCount,
  exporting,
  exportProgress,
  exportFormat,
  exportQuality,
  exportSuccess,
  onExport,
  onFormatChange,
  onQualityChange,
  onCopyToClipboard,
}: Props) {
  const [showSettings, setShowSettings] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!showSettings) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") setShowSettings(false);
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [showSettings]);

  useEffect(() => {
    if (copied) {
      const t = setTimeout(() => setCopied(false), 2000);
      return () => clearTimeout(t);
    }
  }, [copied]);

  if (clipCount === 0) return null;

  if (exporting) {
    return (
      <div className="flex items-center gap-3">
        <span className="text-sm text-zinc-600 dark:text-zinc-300 font-medium">Export...</span>
        <div className="w-48 h-3 bg-zinc-200 dark:bg-zinc-800 rounded-full overflow-hidden border border-zinc-300 dark:border-zinc-700">
          <div
            className="h-full bg-blue-500 rounded-full transition-all duration-300"
            style={{ width: `${exportProgress}%` }}
          />
        </div>
        <span className="text-sm text-zinc-600 dark:text-zinc-300 font-mono tabular-nums">{exportProgress}%</span>
      </div>
    );
  }

  return (
    <div className="relative flex items-center gap-1" data-onboarding-export>
      <button
        onClick={onExport}
        className="px-4 py-2.5 bg-blue-600 hover:bg-blue-500 rounded-l-lg text-sm font-semibold text-white transition-colors cursor-pointer"
      >
        Export {exportFormat.toUpperCase()}
      </button>
      <button
        onClick={() => setShowSettings(!showSettings)}
        className="px-2 py-2.5 bg-blue-600 hover:bg-blue-500 rounded-r-lg text-white transition-colors cursor-pointer border-l border-blue-700"
        title="Export settings"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <polyline points="6 9 12 15 18 9" />
        </svg>
      </button>

      {/* Copy to clipboard button after successful export */}
      {exportSuccess && (
        <button
          onClick={() => {
            onCopyToClipboard(exportSuccess);
            setCopied(true);
          }}
          className="px-3 py-2.5 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-lg text-xs font-medium transition-colors flex items-center gap-1.5"
          title="Copier le chemin dans le presse-papiers"
        >
          {copied ? (
            <svg className="w-3.5 h-3.5 text-green-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          ) : (
            <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
            </svg>
          )}
          {copied ? "Copié" : "Copier"}
        </button>
      )}

      {showSettings && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm animate-fade-in"
          onClick={(e) => {
            if (e.target === e.currentTarget) setShowSettings(false);
          }}
        >
          <div className="bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-2xl shadow-2xl p-5 w-full max-w-xs mx-4 animate-fade-in">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-sm font-semibold text-zinc-800 dark:text-zinc-200">
                Paramètres d'export
              </h3>
              <button
                onClick={() => setShowSettings(false)}
                className="w-6 h-6 flex items-center justify-center rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-700 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 transition-colors"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>

            {/* Format */}
            <div className="mb-4">
              <div className="text-[10px] uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-semibold mb-2">
                Format
              </div>
              <div className="grid grid-cols-2 gap-2">
                {(["mp4", "gif"] as ExportFormat[]).map((f) => (
                  <button
                    key={f}
                    onClick={() => onFormatChange(f)}
                    className={`px-3 py-2 rounded-xl text-sm font-medium transition-all ${
                      exportFormat === f
                        ? "bg-blue-100 dark:bg-blue-900/50 text-blue-600 dark:text-blue-400 ring-2 ring-blue-400 dark:ring-blue-600"
                        : "bg-zinc-50 dark:bg-zinc-700/50 hover:bg-zinc-100 dark:hover:bg-zinc-700 text-zinc-600 dark:text-zinc-300"
                    }`}
                  >
                    {f.toUpperCase()}
                  </button>
                ))}
              </div>
              {exportFormat === "gif" && (
                <p className="text-[10px] text-zinc-400 dark:text-zinc-500 mt-1.5">
                  Résolution réduite, pas de son
                </p>
              )}
            </div>

            {/* Quality */}
            <div>
              <div className="text-[10px] uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-semibold mb-2">
                Qualité
              </div>
              <div className="grid grid-cols-3 gap-2">
                {([
                  { value: "high" as ExportQuality, label: "Haute", desc: "Lent" },
                  { value: "medium" as ExportQuality, label: "Moyenne", desc: "Équilibré" },
                  { value: "low" as ExportQuality, label: "Basse", desc: "Rapide" },
                ]).map((q) => (
                  <button
                    key={q.value}
                    onClick={() => onQualityChange(q.value)}
                    className={`flex flex-col items-center gap-0.5 px-2 py-2 rounded-xl text-xs transition-all ${
                      exportQuality === q.value
                        ? "bg-blue-100 dark:bg-blue-900/50 text-blue-600 dark:text-blue-400 ring-2 ring-blue-400 dark:ring-blue-600"
                        : "bg-zinc-50 dark:bg-zinc-700/50 hover:bg-zinc-100 dark:hover:bg-zinc-700 text-zinc-600 dark:text-zinc-300"
                    }`}
                  >
                    <span className="font-medium">{q.label}</span>
                    <span className="text-[10px] opacity-60">{q.desc}</span>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
