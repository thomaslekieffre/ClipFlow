interface Props {
  seconds: number;
  onChange: (seconds: number) => void;
  disabled?: boolean;
}

export function CountdownSelector({ seconds, onChange, disabled }: Props) {
  const options = [0, 3, 5];
  return (
    <div className="flex items-center gap-1">
      <svg className="w-3.5 h-3.5 text-zinc-400 dark:text-zinc-600 mr-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <circle cx="12" cy="12" r="10" />
        <polyline points="12 6 12 12 16 14" />
      </svg>
      {options.map((n) => (
        <button
          key={n}
          onClick={() => onChange(n)}
          disabled={disabled}
          className={`w-7 h-6 flex items-center justify-center rounded text-[10px] font-mono font-medium transition-all ${
            seconds === n
              ? "bg-blue-500 text-white"
              : "bg-zinc-100 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 hover:bg-zinc-200 dark:hover:bg-zinc-700"
          } ${disabled ? "opacity-40 cursor-not-allowed" : "cursor-pointer"}`}
          title={n === 0 ? "Pas de compte à rebours" : `${n}s avant l'enregistrement`}
        >
          {n}s
        </button>
      ))}
    </div>
  );
}
