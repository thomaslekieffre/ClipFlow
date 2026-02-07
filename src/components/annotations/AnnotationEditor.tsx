import { useState, useRef, useCallback, useEffect } from "react";
import type { Annotation, AnnotationKind, Clip } from "../../lib/types";
import { AnnotationToolbar } from "./AnnotationToolbar";
import { setClipAnnotations, getClipAnnotations } from "../../lib/tauri";

interface Props {
  clip: Clip;
  onClose: () => void;
}

export function AnnotationEditor({ clip, onClose }: Props) {
  const [annotations, setAnnotations] = useState<Annotation[]>([]);
  const [activeTool, setActiveTool] = useState<AnnotationKind | null>(null);
  const [activeColor, setActiveColor] = useState("#ef4444");
  const [drawing, setDrawing] = useState(false);
  const [startPos, setStartPos] = useState({ x: 0, y: 0 });
  const [currentAnnotation, setCurrentAnnotation] = useState<Partial<Annotation> | null>(null);
  const [textInput, setTextInput] = useState<{ x: number; y: number } | null>(null);
  const [textValue, setTextValue] = useState("");
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const textInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    getClipAnnotations(clip.id).then(setAnnotations).catch(console.error);
  }, [clip.id]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [onClose]);

  const getNormalizedPos = useCallback((e: React.MouseEvent) => {
    if (!containerRef.current) return { x: 0, y: 0 };
    const rect = containerRef.current.getBoundingClientRect();
    return {
      x: (e.clientX - rect.left) / rect.width,
      y: (e.clientY - rect.top) / rect.height,
    };
  }, []);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (!activeTool) return;
    const pos = getNormalizedPos(e);
    setDrawing(true);
    setStartPos(pos);

    if (activeTool === "text") {
      setTextInput(pos);
      setTextValue("");
      setDrawing(false);
      setTimeout(() => textInputRef.current?.focus(), 50);
      return;
    }

    setCurrentAnnotation({
      id: crypto.randomUUID(),
      kind: activeTool,
      x: pos.x,
      y: pos.y,
      width: 0,
      height: 0,
      color: activeColor,
      stroke_width: 2,
      text: null,
      points: activeTool === "freehand" ? [[pos.x, pos.y]] : null,
      start_ms: 0,
      end_ms: clip.duration_ms,
    });
  }, [activeTool, activeColor, clip.duration_ms, getNormalizedPos]);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (!drawing || !currentAnnotation) return;
    const pos = getNormalizedPos(e);

    if (currentAnnotation.kind === "freehand") {
      setCurrentAnnotation((prev) => ({
        ...prev!,
        points: [...(prev!.points || []), [pos.x, pos.y]],
      }));
    } else {
      const x = Math.min(startPos.x, pos.x);
      const y = Math.min(startPos.y, pos.y);
      const width = Math.abs(pos.x - startPos.x);
      const height = Math.abs(pos.y - startPos.y);
      setCurrentAnnotation((prev) => ({ ...prev!, x, y, width, height }));
    }
  }, [drawing, currentAnnotation, startPos, getNormalizedPos]);

  const handleMouseUp = useCallback(() => {
    if (!drawing || !currentAnnotation) return;
    setDrawing(false);
    const ann = currentAnnotation as Annotation;
    if (ann.kind === "freehand" || (ann.width > 0.01 && ann.height > 0.01)) {
      setAnnotations((prev) => [...prev, ann]);
    }
    setCurrentAnnotation(null);
  }, [drawing, currentAnnotation]);

  const handleSave = async () => {
    await setClipAnnotations(clip.id, annotations);
    onClose();
  };

  const handleClear = () => {
    setAnnotations([]);
  };

  const commitTextAnnotation = () => {
    if (textInput && textValue.trim()) {
      const ann: Annotation = {
        id: crypto.randomUUID(),
        kind: "text",
        x: textInput.x,
        y: textInput.y,
        width: 0.2,
        height: 0.05,
        color: activeColor,
        stroke_width: 2,
        text: textValue.trim(),
        points: null,
        start_ms: 0,
        end_ms: clip.duration_ms,
      };
      setAnnotations((prev) => [...prev, ann]);
    }
    setTextInput(null);
    setTextValue("");
  };

  const renderAnnotation = (ann: Annotation | Partial<Annotation>, key: string) => {
    const x = (ann.x || 0) * 100;
    const y = (ann.y || 0) * 100;
    const w = (ann.width || 0) * 100;
    const h = (ann.height || 0) * 100;

    switch (ann.kind) {
      case "rectangle":
        return <rect key={key} x={`${x}%`} y={`${y}%`} width={`${w}%`} height={`${h}%`} fill="none" stroke={ann.color} strokeWidth={2} />;
      case "circle":
        return <ellipse key={key} cx={`${x + w / 2}%`} cy={`${y + h / 2}%`} rx={`${w / 2}%`} ry={`${h / 2}%`} fill="none" stroke={ann.color} strokeWidth={2} />;
      case "arrow":
        return <line key={key} x1={`${x}%`} y1={`${y}%`} x2={`${x + w}%`} y2={`${y + h}%`} stroke={ann.color} strokeWidth={2} markerEnd="url(#arrowhead)" />;
      case "text":
        return <text key={key} x={`${x}%`} y={`${y}%`} fill={ann.color} fontSize="14" fontWeight="bold">{ann.text}</text>;
      case "freehand":
        if (ann.points && ann.points.length > 1) {
          const d = ann.points.map((p, i) => `${i === 0 ? "M" : "L"}${p[0] * 100} ${p[1] * 100}`).join(" ");
          return <path key={key} d={d} fill="none" stroke={ann.color} strokeWidth={2} strokeLinecap="round" />;
        }
        return null;
      default:
        return null;
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm animate-fade-in"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
    >
      <div className="bg-white dark:bg-zinc-900 rounded-2xl shadow-2xl overflow-hidden max-w-[80vw] max-h-[85vh] flex flex-col">
        <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-200 dark:border-zinc-700">
          <h3 className="text-sm font-semibold text-zinc-800 dark:text-zinc-200">
            Annotations — Clip {clip.id.slice(0, 8)}
          </h3>
          <div className="flex items-center gap-2">
            <button
              onClick={handleSave}
              className="px-3 py-1.5 bg-blue-500 hover:bg-blue-400 text-white text-xs font-medium rounded-lg transition-colors"
            >
              Sauvegarder
            </button>
            <button
              onClick={onClose}
              className="w-6 h-6 flex items-center justify-center rounded-md hover:bg-zinc-100 dark:hover:bg-zinc-700 text-zinc-400"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
        </div>

        <AnnotationToolbar
          activeTool={activeTool}
          activeColor={activeColor}
          onToolChange={setActiveTool}
          onColorChange={setActiveColor}
          onClear={handleClear}
        />

        <div
          ref={containerRef}
          className="relative bg-black flex-1 cursor-crosshair"
          style={{ aspectRatio: `${clip.region.width}/${clip.region.height}`, maxHeight: "60vh" }}
          onMouseDown={handleMouseDown}
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
        >
          <svg
            ref={svgRef}
            className="absolute inset-0 w-full h-full"
            viewBox="0 0 100 100"
            preserveAspectRatio="none"
          >
            <defs>
              <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto">
                <polygon points="0 0, 10 3.5, 0 7" fill="currentColor" />
              </marker>
            </defs>
            {annotations.map((ann, i) => renderAnnotation(ann, `ann-${i}`))}
            {currentAnnotation && renderAnnotation(currentAnnotation, "current")}
          </svg>

          {/* Inline text input */}
          {textInput && (
            <div
              className="absolute"
              style={{ left: `${textInput.x * 100}%`, top: `${textInput.y * 100}%` }}
            >
              <input
                ref={textInputRef}
                type="text"
                value={textValue}
                onChange={(e) => setTextValue(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") commitTextAnnotation();
                  if (e.key === "Escape") { setTextInput(null); setTextValue(""); }
                }}
                onBlur={commitTextAnnotation}
                className="px-2 py-1 text-sm bg-white dark:bg-zinc-800 border-2 border-blue-500 rounded shadow-lg focus:outline-none min-w-[120px]"
                style={{ color: activeColor }}
                placeholder="Texte..."
              />
            </div>
          )}
        </div>

        <div className="px-4 py-2 border-t border-zinc-200 dark:border-zinc-700 text-[10px] text-zinc-400">
          {annotations.length} annotation{annotations.length !== 1 ? "s" : ""}
        </div>
      </div>
    </div>
  );
}
