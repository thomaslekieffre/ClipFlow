import { useEffect, useRef, useState } from "react";
import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import type { Clip } from "../../lib/types";
import { getThumbnailBase64 } from "../../lib/tauri";

interface Props {
  clip: Clip;
  index: number;
  onDelete: () => void;
}

export function SortableClipCard({ clip, index, onDelete }: Props) {
  const [thumbSrc, setThumbSrc] = useState<string | null>(null);
  const [thumbFailed, setThumbFailed] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const confirmTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: clip.id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
    zIndex: isDragging ? 50 : "auto" as const,
  };

  useEffect(() => {
    let mounted = true;
    getThumbnailBase64(clip.id).then((src) => {
      if (!mounted) return;
      if (src) setThumbSrc(src);
      else setThumbFailed(true);
    }).catch(() => {
      if (mounted) setThumbFailed(true);
    });
    return () => { mounted = false; };
  }, [clip.id]);

  useEffect(() => {
    return () => {
      if (confirmTimer.current) clearTimeout(confirmTimer.current);
    };
  }, []);

  const handleDeleteClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirmDelete) {
      onDelete();
      setConfirmDelete(false);
    } else {
      setConfirmDelete(true);
      confirmTimer.current = setTimeout(() => setConfirmDelete(false), 2000);
    }
  };

  const durationSec = (clip.duration_ms / 1000).toFixed(1);

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="flex-shrink-0 w-44 bg-white dark:bg-zinc-900 rounded-lg border border-zinc-200 dark:border-zinc-800 overflow-hidden group cursor-grab active:cursor-grabbing hover:border-zinc-300 dark:hover:border-zinc-600 hover:shadow-lg hover:shadow-black/5 dark:hover:shadow-black/20 transition-all duration-200"
      {...attributes}
      {...listeners}
    >
      {/* Thumbnail */}
      <div className="h-24 bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center overflow-hidden relative">
        {thumbSrc ? (
          <img
            src={thumbSrc}
            alt={`Clip ${index + 1}`}
            className="w-full h-full object-cover"
            draggable={false}
          />
        ) : thumbFailed ? (
          <svg className="w-8 h-8 text-zinc-300 dark:text-zinc-700" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
            <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18" />
            <line x1="7" y1="2" x2="7" y2="22" />
            <line x1="17" y1="2" x2="17" y2="22" />
            <line x1="2" y1="12" x2="22" y2="12" />
          </svg>
        ) : (
          <div className="w-full h-full bg-zinc-100 dark:bg-zinc-800 animate-pulse" />
        )}
      </div>

      {/* Info bar */}
      <div className="px-3 py-2 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-[10px] text-zinc-500 bg-zinc-100 dark:bg-zinc-800 px-1.5 py-0.5 rounded font-mono">
            {index + 1}
          </span>
          <span className="text-xs text-zinc-500 dark:text-zinc-400 font-mono">{durationSec}s</span>
        </div>
        <button
          onClick={handleDeleteClick}
          className={`transition-all text-xs p-1 ${
            confirmDelete
              ? "opacity-100 text-red-500 dark:text-red-400 font-medium"
              : "opacity-0 group-hover:opacity-100 text-zinc-400 dark:text-zinc-500 hover:text-red-500 dark:hover:text-red-400"
          }`}
          title={confirmDelete ? "Cliquer pour confirmer" : "Supprimer"}
        >
          {confirmDelete ? "Sup ?" : "✕"}
        </button>
      </div>
    </div>
  );
}
