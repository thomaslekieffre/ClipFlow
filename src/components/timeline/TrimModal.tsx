import { useState, useEffect } from "react";
import type { Clip } from "../../lib/types";

interface Props {
  clip: Clip;
  onSave: (trimStartMs: number, trimEndMs: number) => void;
  onClose: () => void;
}

export function TrimModal({ clip, onSave, onClose }: Props) {
  const maxMs = clip.duration_ms;
  const [trimStart, setTrimStart] = useState(clip.trim_start_ms);
  const [trimEnd, setTrimEnd] = useState(clip.trim_end_ms || maxMs);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [onClose]);

  const formatMs = (ms: number) => {
    const s = Math.floor(ms / 1000);
    const tenths = Math.floor((ms % 1000) / 100);
    return `${s}.${tenths}s`;
  };

  const effectiveDuration = trimEnd - trimStart;

  const handleSave = () => {
    const startMs = Math.max(0, trimStart);
    const endMs = trimEnd >= maxMs ? 0 : trimEnd; // 0 means no trim end
    onSave(startMs, endMs);
    onClose();
  };

  const handleReset = () => {
    onSave(0, 0);
    onClose();
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm animate-fade-in"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-2xl shadow-2xl p-5 w-[360px] animate-fade-in">
        {/* Header */}
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-sm font-semibold text-zinc-800 dark:text-zinc-200">
            Trim du clip
          </h3>
          <button
            onClick={onClose}
            className="w-6 h-6 flex items-center justify-center rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-700 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 transition-colors"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        {/* Duration info */}
        <div className="text-xs text-zinc-500 dark:text-zinc-400 mb-4 flex items-center justify-between">
          <span>Durée originale : {formatMs(maxMs)}</span>
          <span className={effectiveDuration < maxMs ? "text-blue-500 font-medium" : ""}>
            → {formatMs(effectiveDuration)}
          </span>
        </div>

        {/* Visual range */}
        <div className="relative h-8 bg-zinc-100 dark:bg-zinc-700 rounded-lg mb-4 overflow-hidden">
          <div
            className="absolute h-full bg-blue-500/30 dark:bg-blue-500/40 border-x-2 border-blue-500"
            style={{
              left: `${(trimStart / maxMs) * 100}%`,
              width: `${((trimEnd - trimStart) / maxMs) * 100}%`,
            }}
          />
        </div>

        {/* Start slider */}
        <div className="mb-3">
          <div className="flex items-center justify-between mb-1">
            <label className="text-[10px] uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-semibold">
              Début
            </label>
            <span className="text-xs font-mono text-zinc-500 dark:text-zinc-400">{formatMs(trimStart)}</span>
          </div>
          <input
            type="range"
            min={0}
            max={maxMs}
            step={100}
            value={trimStart}
            onChange={(e) => {
              const v = Number(e.target.value);
              setTrimStart(Math.min(v, trimEnd - 100));
            }}
            className="w-full accent-blue-500"
          />
        </div>

        {/* End slider */}
        <div className="mb-4">
          <div className="flex items-center justify-between mb-1">
            <label className="text-[10px] uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-semibold">
              Fin
            </label>
            <span className="text-xs font-mono text-zinc-500 dark:text-zinc-400">{formatMs(trimEnd)}</span>
          </div>
          <input
            type="range"
            min={0}
            max={maxMs}
            step={100}
            value={trimEnd}
            onChange={(e) => {
              const v = Number(e.target.value);
              setTrimEnd(Math.max(v, trimStart + 100));
            }}
            className="w-full accent-blue-500"
          />
        </div>

        {/* Actions */}
        <div className="flex items-center gap-2">
          <button
            onClick={handleReset}
            className="px-3 py-2 text-xs text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200 transition-colors"
          >
            Reset
          </button>
          <div className="flex-1" />
          <button
            onClick={onClose}
            className="px-4 py-2 bg-zinc-100 dark:bg-zinc-700 hover:bg-zinc-200 dark:hover:bg-zinc-600 rounded-lg text-xs font-medium transition-colors"
          >
            Annuler
          </button>
          <button
            onClick={handleSave}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded-lg text-xs font-semibold text-white transition-colors"
          >
            Appliquer
          </button>
        </div>
      </div>
    </div>
  );
}
