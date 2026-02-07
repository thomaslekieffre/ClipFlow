import { useEffect, useState } from "react";
import type { ProjectSummary } from "../../lib/types";

interface Props {
  projects: ProjectSummary[];
  currentProjectId: string | null;
  onLoad: (projectId: string) => void;
  onDelete: (projectId: string) => void;
  onClose: () => void;
}

export function ProjectListModal({ projects, currentProjectId, onLoad, onDelete, onClose }: Props) {
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [onClose]);

  const formatDate = (dateStr: string) => {
    try {
      const d = new Date(dateStr);
      return d.toLocaleDateString("fr-FR", { day: "2-digit", month: "short", year: "numeric", hour: "2-digit", minute: "2-digit" });
    } catch {
      return dateStr;
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm animate-fade-in"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="bg-white dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-2xl shadow-2xl p-5 w-full max-w-md mx-4 max-h-[70vh] flex flex-col animate-fade-in">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-sm font-semibold text-zinc-800 dark:text-zinc-200">
            Projets sauvegardés
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

        {projects.length === 0 ? (
          <p className="text-sm text-zinc-400 dark:text-zinc-500 text-center py-8">
            Aucun projet sauvegardé
          </p>
        ) : (
          <div className="space-y-2 overflow-y-auto flex-1">
            {projects.map((project) => (
              <div
                key={project.id}
                className={`px-3 py-2.5 rounded-xl transition-colors ${
                  project.id === currentProjectId
                    ? "bg-blue-50 dark:bg-blue-900/30 border border-blue-200 dark:border-blue-800"
                    : "bg-zinc-50 dark:bg-zinc-700/50 hover:bg-zinc-100 dark:hover:bg-zinc-700 border border-transparent"
                }`}
              >
                <div className="flex items-center justify-between">
                  <button
                    onClick={() => onLoad(project.id)}
                    className="flex-1 text-left"
                  >
                    <div className="text-sm font-medium text-zinc-800 dark:text-zinc-200">
                      {project.name}
                    </div>
                    <div className="text-[10px] text-zinc-400 dark:text-zinc-500 mt-0.5">
                      {project.clip_count} clip{project.clip_count > 1 ? "s" : ""} · {(project.total_duration_ms / 1000).toFixed(1)}s · {formatDate(project.updated_at)}
                    </div>
                  </button>
                  <button
                    onClick={() => {
                      if (confirmDeleteId === project.id) {
                        onDelete(project.id);
                        setConfirmDeleteId(null);
                      } else {
                        setConfirmDeleteId(project.id);
                        setTimeout(() => setConfirmDeleteId(null), 2000);
                      }
                    }}
                    className={`ml-2 text-xs p-1 transition-colors ${
                      confirmDeleteId === project.id
                        ? "text-red-500 font-medium"
                        : "text-zinc-400 hover:text-red-500"
                    }`}
                    title={confirmDeleteId === project.id ? "Confirmer suppression" : "Supprimer"}
                  >
                    {confirmDeleteId === project.id ? "Sup ?" : "✕"}
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
