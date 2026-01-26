import { useState, useEffect } from "react";
import type { Transition, TransitionType } from "../../lib/types";

interface Props {
  transition: Transition | undefined;
  onChange: (type: TransitionType) => void;
}

interface TransitionInfo {
  type: TransitionType;
  label: string;
  icon: string;
  group: string;
}

const TRANSITIONS: TransitionInfo[] = [
  // Fades
  { type: "fade", label: "Fade", icon: "◐", group: "Fondu" },
  { type: "fadeblack", label: "Noir", icon: "●", group: "Fondu" },
  { type: "fadewhite", label: "Blanc", icon: "○", group: "Fondu" },
  { type: "dissolve", label: "Dissolve", icon: "◑", group: "Fondu" },
  // Slides
  { type: "slide", label: "← Slide", icon: "◁", group: "Slide" },
  { type: "slideright", label: "Slide →", icon: "▷", group: "Slide" },
  { type: "slideup", label: "Slide ↑", icon: "△", group: "Slide" },
  { type: "slidedown", label: "Slide ↓", icon: "▽", group: "Slide" },
  // Wipes
  { type: "wipeleft", label: "← Wipe", icon: "◧", group: "Wipe" },
  { type: "wiperight", label: "Wipe →", icon: "◨", group: "Wipe" },
  { type: "wipeup", label: "Wipe ↑", icon: "⬒", group: "Wipe" },
  { type: "wipedown", label: "Wipe ↓", icon: "⬓", group: "Wipe" },
  // Effects
  { type: "zoom", label: "Zoom", icon: "⊕", group: "Effet" },
  { type: "pixelize", label: "Pixel", icon: "▦", group: "Effet" },
  { type: "circleopen", label: "Iris Open", icon: "◎", group: "Effet" },
  { type: "circleclose", label: "Iris Close", icon: "◉", group: "Effet" },
  { type: "radial", label: "Radial", icon: "✶", group: "Effet" },
  { type: "smoothleft", label: "← Smooth", icon: "≪", group: "Effet" },
  { type: "smoothright", label: "Smooth →", icon: "≫", group: "Effet" },
];

const LABEL_MAP = Object.fromEntries(TRANSITIONS.map((t) => [t.type, t.label]));
const ICON_MAP = Object.fromEntries(TRANSITIONS.map((t) => [t.type, t.icon]));

// Group transitions
const GROUPS = TRANSITIONS.reduce<Record<string, TransitionInfo[]>>(
  (acc, t) => {
    if (!acc[t.group]) acc[t.group] = [];
    acc[t.group].push(t);
    return acc;
  },
  {},
);

export function TransitionIcon({ transition, onChange }: Props) {
  const [open, setOpen] = useState(false);
  const currentType = transition?.transition_type ?? "fade";

  // Close on Escape
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [open]);

  return (
    <>
      <div className="relative flex-shrink-0 mx-1">
        <button
          onClick={() => setOpen(true)}
          className="w-8 h-8 flex items-center justify-center rounded-full bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 border border-zinc-200 dark:border-zinc-700 text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200 text-sm transition-all hover:scale-110"
          title={`Transition: ${LABEL_MAP[currentType] ?? currentType}`}
        >
          {ICON_MAP[currentType] ?? "◐"}
        </button>
      </div>

      {/* Modal overlay */}
      {open && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm animate-fade-in"
          onClick={(e) => {
            if (e.target === e.currentTarget) setOpen(false);
          }}
        >
          <div className="bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-2xl shadow-2xl p-5 w-[340px] animate-fade-in">
            {/* Header */}
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-sm font-semibold text-zinc-800 dark:text-zinc-200">
                Transition
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

            {/* Groups */}
            {Object.entries(GROUPS).map(([groupName, items]) => (
              <div key={groupName} className="mb-3 last:mb-0">
                <div className="text-[10px] uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-semibold mb-1.5 px-0.5">
                  {groupName}
                </div>
                <div className="grid grid-cols-4 gap-1.5">
                  {items.map((t) => (
                    <button
                      key={t.type}
                      onClick={() => {
                        onChange(t.type);
                        setOpen(false);
                      }}
                      className={`flex flex-col items-center gap-1 px-1.5 py-2 rounded-xl text-xs transition-all hover:scale-105 cursor-pointer ${
                        currentType === t.type
                          ? "bg-blue-100 dark:bg-blue-900/50 text-blue-600 dark:text-blue-400 ring-2 ring-blue-400 dark:ring-blue-600"
                          : "bg-zinc-50 dark:bg-zinc-700/50 hover:bg-zinc-100 dark:hover:bg-zinc-700 text-zinc-600 dark:text-zinc-300"
                      }`}
                    >
                      <span className="text-lg leading-none">{t.icon}</span>
                      <span className="text-[10px] leading-tight truncate w-full text-center font-medium">
                        {t.label}
                      </span>
                    </button>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </>
  );
}
