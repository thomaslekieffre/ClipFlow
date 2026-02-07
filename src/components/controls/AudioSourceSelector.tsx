import type { AudioSource } from "../../lib/types";

interface Props {
  audioSource: AudioSource;
  onChange: (source: AudioSource) => void;
  disabled?: boolean;
}

const sources: { value: AudioSource; label: string; icon: string }[] = [
  { value: "none", label: "Muet", icon: "M" },
  { value: "system", label: "Système", icon: "S" },
  { value: "microphone", label: "Micro", icon: "Mi" },
  { value: "both", label: "Les deux", icon: "+" },
];

export function AudioSourceSelector({ audioSource, onChange, disabled }: Props) {
  return (
    <div className="flex items-center gap-1">
      {sources.map((s) => (
        <button
          key={s.value}
          onClick={() => onChange(s.value)}
          disabled={disabled}
          className={`px-2 py-1 rounded text-[10px] font-medium transition-all ${
            audioSource === s.value
              ? "bg-blue-500 text-white"
              : "bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700"
          } ${disabled ? "opacity-40 cursor-not-allowed" : "cursor-pointer"}`}
          title={s.label}
        >
          {s.icon === "M" ? (
            <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="1" y1="1" x2="23" y2="23" />
              <path d="M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6" />
              <path d="M17 16.95A7 7 0 0 1 5 12v-2m14 0v2c0 .84-.16 1.65-.45 2.39" />
              <line x1="12" y1="19" x2="12" y2="23" />
              <line x1="8" y1="23" x2="16" y2="23" />
            </svg>
          ) : s.icon === "S" ? (
            <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
              <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07" />
            </svg>
          ) : s.icon === "Mi" ? (
            <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
              <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
              <line x1="12" y1="19" x2="12" y2="23" />
              <line x1="8" y1="23" x2="16" y2="23" />
            </svg>
          ) : (
            <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
              <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
              <circle cx="18" cy="18" r="3" />
              <path d="M18 15v-1a3 3 0 0 0-3-3" />
            </svg>
          )}
        </button>
      ))}
    </div>
  );
}
