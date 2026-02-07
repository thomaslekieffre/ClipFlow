import { useState, useEffect } from "react";
import type { Subtitle, SubtitlePosition } from "../../lib/types";
import { getSubtitles, setSubtitles } from "../../lib/tauri";

interface Props {
  totalDurationMs: number;
  onClose: () => void;
}

export function SubtitleEditor({ totalDurationMs, onClose }: Props) {
  const [subs, setSubs] = useState<Subtitle[]>([]);
  const [editingId, setEditingId] = useState<string | null>(null);

  useEffect(() => {
    getSubtitles().then(setSubs).catch(console.error);
  }, []);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [onClose]);

  const addSubtitle = () => {
    const newSub: Subtitle = {
      id: crypto.randomUUID(),
      text: "Nouveau sous-titre",
      start_ms: 0,
      end_ms: Math.min(3000, totalDurationMs),
      position: "bottom",
      font_size: 32,
      color: "#ffffff",
    };
    setSubs((prev) => [...prev, newSub]);
    setEditingId(newSub.id);
  };

  const updateSubtitle = (id: string, updates: Partial<Subtitle>) => {
    setSubs((prev) => prev.map((s) => (s.id === id ? { ...s, ...updates } : s)));
  };

  const deleteSubtitle = (id: string) => {
    setSubs((prev) => prev.filter((s) => s.id !== id));
    if (editingId === id) setEditingId(null);
  };

  const handleSave = async () => {
    await setSubtitles(subs);
    onClose();
  };

  const formatTime = (ms: number) => {
    const s = Math.floor(ms / 1000);
    const m = Math.floor(s / 60);
    const rest = s % 60;
    return `${m}:${rest.toString().padStart(2, "0")}`;
  };

  const parseTime = (str: string): number | null => {
    const parts = str.split(":");
    if (parts.length !== 2) return null;
    const m = parseInt(parts[0], 10);
    const s = parseInt(parts[1], 10);
    if (isNaN(m) || isNaN(s)) return null;
    return (m * 60 + s) * 1000;
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm animate-fade-in"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-2xl shadow-2xl p-5 w-full max-w-md mx-4 max-h-[80vh] flex flex-col animate-fade-in">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-sm font-semibold text-zinc-800 dark:text-zinc-200">
            Sous-titres
          </h3>
          <div className="flex items-center gap-2">
            <button
              onClick={handleSave}
              className="px-3 py-1.5 bg-blue-500 hover:bg-blue-400 text-white text-xs font-medium rounded-lg transition-colors"
            >
              Sauvegarder
            </button>
            <button
              onClick={onClose}
              className="w-6 h-6 flex items-center justify-center rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-700 text-zinc-400"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
        </div>

        {/* Mini timeline */}
        {subs.length > 0 && totalDurationMs > 0 && (
          <div className="h-6 bg-zinc-100 dark:bg-zinc-700 rounded-lg mb-3 relative overflow-hidden cursor-pointer">
            {subs.map((sub) => {
              const left = (sub.start_ms / totalDurationMs) * 100;
              const width = ((sub.end_ms - sub.start_ms) / totalDurationMs) * 100;
              return (
                <div
                  key={sub.id}
                  onClick={() => setEditingId(sub.id)}
                  className={`absolute h-full rounded transition-colors ${
                    editingId === sub.id
                      ? "bg-blue-400/60 dark:bg-blue-500/50"
                      : "bg-blue-300/40 dark:bg-blue-600/30 hover:bg-blue-300/60"
                  }`}
                  style={{ left: `${left}%`, width: `${Math.max(width, 1)}%` }}
                  title={sub.text}
                />
              );
            })}
          </div>
        )}

        <div className="space-y-2 overflow-y-auto flex-1 mb-3">
          {subs.map((sub) => (
            <div
              key={sub.id}
              className={`px-3 py-2.5 rounded-xl border transition-colors ${
                editingId === sub.id
                  ? "border-blue-400 dark:border-blue-600 bg-blue-50 dark:bg-blue-900/20"
                  : "border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-700/50"
              }`}
              onClick={() => setEditingId(sub.id)}
            >
              {editingId === sub.id ? (
                <div className="space-y-3">
                  {/* Text input */}
                  <input
                    type="text"
                    value={sub.text}
                    onChange={(e) => updateSubtitle(sub.id, { text: e.target.value })}
                    className="w-full px-2 py-1 text-sm bg-white dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-600 rounded focus:outline-none focus:ring-1 focus:ring-blue-500"
                    autoFocus
                  />

                  {/* Start time */}
                  <div>
                    <div className="flex items-center justify-between mb-1">
                      <label className="text-[10px] uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-semibold">
                        Début
                      </label>
                      <input
                        type="text"
                        value={formatTime(sub.start_ms)}
                        onChange={(e) => {
                          const ms = parseTime(e.target.value);
                          if (ms !== null) updateSubtitle(sub.id, { start_ms: Math.min(ms, sub.end_ms - 100) });
                        }}
                        className="w-14 text-xs font-mono text-center bg-white dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-600 rounded px-1 py-0.5 focus:outline-none focus:ring-1 focus:ring-blue-500"
                      />
                    </div>
                    <input
                      type="range"
                      min={0}
                      max={totalDurationMs}
                      step={100}
                      value={sub.start_ms}
                      onChange={(e) => updateSubtitle(sub.id, { start_ms: Math.min(Number(e.target.value), sub.end_ms - 100) })}
                      className="w-full accent-blue-500"
                    />
                  </div>

                  {/* End time */}
                  <div>
                    <div className="flex items-center justify-between mb-1">
                      <label className="text-[10px] uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-semibold">
                        Fin
                      </label>
                      <input
                        type="text"
                        value={formatTime(sub.end_ms)}
                        onChange={(e) => {
                          const ms = parseTime(e.target.value);
                          if (ms !== null) updateSubtitle(sub.id, { end_ms: Math.max(ms, sub.start_ms + 100) });
                        }}
                        className="w-14 text-xs font-mono text-center bg-white dark:bg-zinc-800 border border-zinc-300 dark:border-zinc-600 rounded px-1 py-0.5 focus:outline-none focus:ring-1 focus:ring-blue-500"
                      />
                    </div>
                    <input
                      type="range"
                      min={0}
                      max={totalDurationMs}
                      step={100}
                      value={sub.end_ms}
                      onChange={(e) => updateSubtitle(sub.id, { end_ms: Math.max(Number(e.target.value), sub.start_ms + 100) })}
                      className="w-full accent-blue-500"
                    />
                  </div>

                  {/* Position buttons */}
                  <div className="flex items-center gap-2">
                    <span className="text-[10px] text-zinc-400">Position :</span>
                    {(["top", "center", "bottom"] as SubtitlePosition[]).map((pos) => (
                      <button
                        key={pos}
                        onClick={(e) => { e.stopPropagation(); updateSubtitle(sub.id, { position: pos }); }}
                        className={`px-1.5 py-0.5 rounded text-[9px] font-medium ${
                          sub.position === pos
                            ? "bg-blue-500 text-white"
                            : "bg-zinc-200 dark:bg-zinc-600 text-zinc-600 dark:text-zinc-300"
                        }`}
                      >
                        {pos === "top" ? "Haut" : pos === "center" ? "Centre" : "Bas"}
                      </button>
                    ))}
                  </div>

                  {/* Font size slider + Color picker */}
                  <div className="flex items-center gap-3">
                    <div className="flex-1">
                      <div className="flex items-center justify-between mb-1">
                        <span className="text-[10px] text-zinc-400">Taille</span>
                        <span className="text-[10px] font-mono text-zinc-500">{sub.font_size}px</span>
                      </div>
                      <input
                        type="range"
                        min={16}
                        max={72}
                        step={2}
                        value={sub.font_size}
                        onChange={(e) => updateSubtitle(sub.id, { font_size: Number(e.target.value) })}
                        className="w-full accent-blue-500"
                      />
                    </div>
                    <div className="flex flex-col items-center gap-1">
                      <span className="text-[10px] text-zinc-400">Couleur</span>
                      <input
                        type="color"
                        value={sub.color}
                        onChange={(e) => updateSubtitle(sub.id, { color: e.target.value })}
                        className="w-7 h-7 rounded cursor-pointer border border-zinc-300 dark:border-zinc-600 p-0"
                      />
                    </div>
                  </div>

                  {/* Delete button */}
                  <div className="flex justify-end">
                    <button
                      onClick={(e) => { e.stopPropagation(); deleteSubtitle(sub.id); }}
                      className="w-7 h-7 flex items-center justify-center bg-red-50 dark:bg-red-900/30 hover:bg-red-100 dark:hover:bg-red-900/50 rounded-lg transition-colors"
                      title="Supprimer"
                    >
                      <svg className="w-3.5 h-3.5 text-red-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <polyline points="3 6 5 6 21 6" />
                        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                      </svg>
                    </button>
                  </div>
                </div>
              ) : (
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2 flex-1 min-w-0">
                    <div
                      className="w-3 h-3 rounded-sm shrink-0 border border-zinc-300 dark:border-zinc-600"
                      style={{ backgroundColor: sub.color }}
                    />
                    <span className="text-sm text-zinc-700 dark:text-zinc-300 truncate">{sub.text}</span>
                  </div>
                  <div className="flex items-center gap-2 ml-2 shrink-0">
                    <span className="text-[9px] text-zinc-400 font-mono">{sub.font_size}px</span>
                    <span className="text-[10px] text-zinc-400 font-mono">
                      {formatTime(sub.start_ms)}-{formatTime(sub.end_ms)}
                    </span>
                  </div>
                </div>
              )}
            </div>
          ))}

          {subs.length === 0 && (
            <div className="flex flex-col items-center py-8 text-zinc-400 dark:text-zinc-500">
              <svg className="w-10 h-10 mb-2 text-zinc-300 dark:text-zinc-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                <rect x="2" y="4" width="20" height="16" rx="2" />
                <line x1="6" y1="14" x2="18" y2="14" />
                <line x1="6" y1="18" x2="14" y2="18" />
              </svg>
              <p className="text-sm font-medium">Aucun sous-titre</p>
              <p className="text-[11px] text-zinc-400 dark:text-zinc-600 mt-1">
                Ajoutez du texte affichable pendant la lecture
              </p>
            </div>
          )}
        </div>

        <button
          onClick={addSubtitle}
          className="w-full py-2 border-2 border-dashed border-zinc-300 dark:border-zinc-600 rounded-xl text-xs text-zinc-500 hover:border-blue-400 hover:text-blue-500 transition-colors"
        >
          + Ajouter un sous-titre
        </button>
      </div>
    </div>
  );
}
