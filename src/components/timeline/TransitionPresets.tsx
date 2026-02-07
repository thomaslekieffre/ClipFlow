import { useState, useEffect } from "react";

interface Props {
  onApply: (preset: string) => void;
  hasTransitions: boolean;
}

const presets = [
  {
    id: "professional",
    label: "Professionnel",
    desc: "Fondus doux et élégants",
    transitions: "Fade",
    color: "blue",
  },
  {
    id: "dynamic",
    label: "Dynamique",
    desc: "Slides et zooms variés",
    transitions: "Slide + Zoom",
    color: "orange",
  },
  {
    id: "minimal",
    label: "Minimal",
    desc: "Coupe directe, sans effet",
    transitions: "Cut",
    color: "zinc",
  },
  {
    id: "creative",
    label: "Créatif",
    desc: "Effets variés et originaux",
    transitions: "Circle, Pixelize, Radial...",
    color: "purple",
  },
];

export function TransitionPresets({ onApply, hasTransitions }: Props) {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [open]);

  if (!hasTransitions) return null;

  return (
    <>
      <button
        onClick={() => setOpen(true)}
        className="px-2.5 py-1 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-md text-[10px] font-medium text-zinc-500 dark:text-zinc-400 transition-colors"
        title="Appliquer un preset de transitions"
      >
        Presets
      </button>

      {open && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm animate-fade-in"
          onClick={(e) => {
            if (e.target === e.currentTarget) setOpen(false);
          }}
        >
          <div className="bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-2xl shadow-2xl p-5 w-full max-w-sm mx-4 animate-fade-in">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-sm font-semibold text-zinc-800 dark:text-zinc-200">
                Presets de transitions
              </h3>
              <button
                onClick={() => setOpen(false)}
                className="w-6 h-6 flex items-center justify-center rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-700 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 transition-colors"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>

            <div className="space-y-2">
              {presets.map((preset) => (
                <button
                  key={preset.id}
                  onClick={() => {
                    onApply(preset.id);
                    setOpen(false);
                  }}
                  className="w-full text-left px-3 py-2.5 rounded-xl bg-zinc-50 dark:bg-zinc-700/50 hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors group"
                >
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium text-zinc-800 dark:text-zinc-200">
                      {preset.label}
                    </span>
                    <span className="text-[10px] text-zinc-400 dark:text-zinc-500 font-mono">
                      {preset.transitions}
                    </span>
                  </div>
                  <p className="text-[11px] text-zinc-500 dark:text-zinc-400 mt-0.5">
                    {preset.desc}
                  </p>
                </button>
              ))}
            </div>
          </div>
        </div>
      )}
    </>
  );
}
