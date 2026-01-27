import type { RecordingState } from "../../lib/types";

interface Props {
  state: RecordingState;
}

export function StatusIndicator({ state }: Props) {
  if (state === "recording") {
    return (
      <div className="flex items-center gap-2">
        <span className="relative flex h-2.5 w-2.5">
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-500 opacity-75" />
          <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-red-500" />
        </span>
        <span className="text-sm text-red-500 dark:text-red-400 font-medium">REC</span>
      </div>
    );
  }

  if (state === "paused") {
    return (
      <div className="flex items-center gap-2">
        <span className="h-2.5 w-2.5 rounded-full bg-yellow-500 animate-pulse" />
        <span className="text-sm text-yellow-500 dark:text-yellow-400 font-medium">PAUSE</span>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2">
      <span className="h-2.5 w-2.5 rounded-full bg-emerald-500" />
      <span className="text-sm text-emerald-600 dark:text-emerald-500">Prêt</span>
    </div>
  );
}
