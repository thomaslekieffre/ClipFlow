import type { AnnotationKind } from "../../lib/types";

interface Props {
  activeTool: AnnotationKind | null;
  activeColor: string;
  onToolChange: (tool: AnnotationKind | null) => void;
  onColorChange: (color: string) => void;
  onClear: () => void;
}

const tools: { kind: AnnotationKind; label: string }[] = [
  { kind: "arrow", label: "Flèche" },
  { kind: "rectangle", label: "Rectangle" },
  { kind: "circle", label: "Cercle" },
  { kind: "text", label: "Texte" },
  { kind: "freehand", label: "Libre" },
];

const colors = ["#ef4444", "#3b82f6", "#22c55e", "#f59e0b", "#ffffff", "#000000"];

export function AnnotationToolbar({ activeTool, activeColor, onToolChange, onColorChange, onClear }: Props) {
  return (
    <div className="flex items-center gap-2 p-2 bg-zinc-100 dark:bg-zinc-800 rounded-lg">
      {/* Tools */}
      <div className="flex items-center gap-1">
        {tools.map((tool) => (
          <button
            key={tool.kind}
            onClick={() => onToolChange(activeTool === tool.kind ? null : tool.kind)}
            className={`px-2 py-1 rounded text-[10px] font-medium transition-all ${
              activeTool === tool.kind
                ? "bg-blue-500 text-white"
                : "bg-white dark:bg-zinc-700 text-zinc-600 dark:text-zinc-300 hover:bg-zinc-200 dark:hover:bg-zinc-600"
            }`}
            title={tool.label}
          >
            {tool.kind === "arrow" && (
              <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <line x1="5" y1="12" x2="19" y2="12" /><polyline points="12 5 19 12 12 19" />
              </svg>
            )}
            {tool.kind === "rectangle" && (
              <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="3" y="3" width="18" height="18" rx="2" />
              </svg>
            )}
            {tool.kind === "circle" && (
              <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="12" r="10" />
              </svg>
            )}
            {tool.kind === "text" && (
              <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polyline points="4 7 4 4 20 4 20 7" /><line x1="9.5" y1="20" x2="14.5" y2="20" /><line x1="12" y1="4" x2="12" y2="20" />
              </svg>
            )}
            {tool.kind === "freehand" && (
              <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M12 19l7-7 3 3-7 7-3-3z" /><path d="M18 13l-1.5-7.5L2 2l3.5 14.5L13 18l5-5z" />
              </svg>
            )}
          </button>
        ))}
      </div>

      <div className="w-px h-5 bg-zinc-300 dark:bg-zinc-600" />

      {/* Colors */}
      <div className="flex items-center gap-1">
        {colors.map((color) => (
          <button
            key={color}
            onClick={() => onColorChange(color)}
            className={`w-4 h-4 rounded-full border-2 transition-all ${
              activeColor === color ? "border-blue-500 scale-125" : "border-zinc-300 dark:border-zinc-600"
            }`}
            style={{ backgroundColor: color }}
            title={color}
          />
        ))}
      </div>

      <div className="w-px h-5 bg-zinc-300 dark:bg-zinc-600" />

      <button
        onClick={onClear}
        className="px-2 py-1 rounded text-[10px] font-medium text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
      >
        Effacer tout
      </button>
    </div>
  );
}
