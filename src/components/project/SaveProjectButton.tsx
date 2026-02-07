import { useState } from "react";

interface Props {
  onSave: (name: string) => void;
  onListOpen: () => void;
  hasClips: boolean;
  disabled?: boolean;
}

export function SaveProjectButton({ onSave, onListOpen, hasClips, disabled }: Props) {
  const [showInput, setShowInput] = useState(false);
  const [name, setName] = useState("");

  const handleSave = () => {
    if (name.trim()) {
      onSave(name.trim());
      setName("");
      setShowInput(false);
    }
  };

  return (
    <div className="flex items-center gap-1">
      <button
        onClick={() => {
          if (showInput) {
            handleSave();
          } else {
            setShowInput(true);
            setName(`Projet ${new Date().toLocaleDateString("fr-FR")}`);
          }
        }}
        disabled={disabled || !hasClips}
        className="px-2.5 py-1.5 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 disabled:opacity-40 disabled:cursor-not-allowed rounded-lg text-xs font-medium transition-colors flex items-center gap-1.5"
        title="Sauvegarder le projet"
      >
        <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
          <polyline points="17 21 17 13 7 13 7 21" />
          <polyline points="7 3 7 8 15 8" />
        </svg>
        Sauver
      </button>

      {showInput && (
        <div className="flex items-center gap-1">
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleSave();
              if (e.key === "Escape") setShowInput(false);
            }}
            className="px-2 py-1 text-xs bg-white dark:bg-zinc-900 border border-zinc-300 dark:border-zinc-600 rounded-md w-36 focus:outline-none focus:ring-1 focus:ring-blue-500"
            placeholder="Nom du projet"
            autoFocus
          />
          <button
            onClick={() => setShowInput(false)}
            className="text-zinc-400 hover:text-zinc-600 text-xs"
          >
            ✕
          </button>
        </div>
      )}

      <button
        onClick={onListOpen}
        className="px-2.5 py-1.5 bg-zinc-100 dark:bg-zinc-800 hover:bg-zinc-200 dark:hover:bg-zinc-700 rounded-lg text-xs font-medium transition-colors flex items-center gap-1.5"
        title="Ouvrir un projet"
      >
        <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
        </svg>
        Ouvrir
      </button>
    </div>
  );
}
