interface Props {
  clipCount: number;
  exporting: boolean;
  exportProgress: number;
  onExport: () => void;
}

export function ExportButton({ clipCount, exporting, exportProgress, onExport }: Props) {
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
    <button
      onClick={onExport}
      className="px-4 py-2.5 bg-blue-600 hover:bg-blue-500 rounded-lg text-sm font-semibold text-white transition-colors cursor-pointer"
    >
      Export MP4
    </button>
  );
}
