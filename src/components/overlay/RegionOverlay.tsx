import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export function RegionOverlay() {
  const [drawing, setDrawing] = useState(false);
  const [startPos, setStartPos] = useState({ x: 0, y: 0 });
  const [rect, setRect] = useState<Rect | null>(null);
  const [confirmed, setConfirmed] = useState(false);
  const overlayRef = useRef<HTMLDivElement>(null);

  // Auto-focus the overlay so keyboard events work
  useEffect(() => {
    overlayRef.current?.focus();
  }, []);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      if (confirmed) return;
      // Ignore clicks on buttons
      if ((e.target as HTMLElement).closest("button")) return;
      setDrawing(true);
      setStartPos({ x: e.clientX, y: e.clientY });
      setRect({ x: e.clientX, y: e.clientY, width: 0, height: 0 });
    },
    [confirmed],
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (!drawing) return;
      const x = Math.min(startPos.x, e.clientX);
      const y = Math.min(startPos.y, e.clientY);
      const width = Math.abs(e.clientX - startPos.x);
      const height = Math.abs(e.clientY - startPos.y);
      setRect({ x, y, width, height });
    },
    [drawing, startPos],
  );

  const handleMouseUp = useCallback(() => {
    if (!drawing) return;
    setDrawing(false);
    if (rect && rect.width > 10 && rect.height > 10) {
      setConfirmed(true);
    } else {
      setRect(null);
    }
  }, [drawing, rect]);

  const closeOverlay = useCallback(async () => {
    try {
      const win = getCurrentWebviewWindow();
      await win.close();
    } catch {
      // fallback
      await invoke("close_region_selector");
    }
  }, []);

  const handleConfirm = useCallback(async () => {
    if (!rect) return;
    const win = getCurrentWebviewWindow();

    // Get window position (physical pixels) and scale factor
    let offsetX = 0;
    let offsetY = 0;
    let scale = 1;
    try {
      const pos = await win.outerPosition();
      offsetX = pos.x;
      offsetY = pos.y;
      scale = await win.scaleFactor();
    } catch {
      // fallback
    }

    // clientX/clientY are in CSS (logical) pixels
    // outerPosition and gdigrab use physical pixels
    // Convert logical → physical by multiplying by scale factor
    const region = {
      x: Math.round(rect.x * scale + offsetX),
      y: Math.round(rect.y * scale + offsetY),
      width: Math.round((rect.width * scale) / 2) * 2,
      height: Math.round((rect.height * scale) / 2) * 2,
    };
    await invoke("set_capture_region", { region });
    await emit("region-selected", region);
    await closeOverlay();
  }, [rect, closeOverlay]);

  const handleReset = useCallback(() => {
    setRect(null);
    setConfirmed(false);
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (confirmed) {
          handleReset();
        } else {
          closeOverlay();
        }
      } else if (e.key === "Enter" && confirmed) {
        handleConfirm();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [confirmed, closeOverlay, handleConfirm, handleReset]);

  return (
    <div
      ref={overlayRef}
      tabIndex={-1}
      className="fixed inset-0 outline-none"
      style={{ background: "rgba(0,0,0,0.45)", cursor: confirmed ? "default" : "crosshair" }}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
    >
      {/* Instructions */}
      {!rect && (
        <div className="absolute top-8 left-1/2 -translate-x-1/2 bg-black/80 text-white px-5 py-2.5 rounded-lg text-sm pointer-events-none z-50">
          Dessine un rectangle pour sélectionner la zone · ESC pour annuler
        </div>
      )}

      {/* Selection rectangle — cut-out effect */}
      {rect && rect.width > 0 && rect.height > 0 && (
        <div
          className="absolute border-2 border-blue-400 pointer-events-none"
          style={{
            left: rect.x,
            top: rect.y,
            width: rect.width,
            height: rect.height,
            background: "transparent",
            boxShadow: "0 0 0 9999px rgba(0,0,0,0.45)",
            zIndex: 10,
          }}
        >
          {/* Dimensions label (approximate, exact after confirm) */}
          <div className="absolute -top-7 left-0 bg-blue-500 text-white text-xs px-2 py-0.5 rounded font-mono whitespace-nowrap">
            {Math.round(rect.width)} × {Math.round(rect.height)}
          </div>

          {/* Corner handles */}
          {confirmed && (
            <>
              <div className="absolute -top-1 -left-1 w-2.5 h-2.5 bg-blue-400 border border-white" />
              <div className="absolute -top-1 -right-1 w-2.5 h-2.5 bg-blue-400 border border-white" />
              <div className="absolute -bottom-1 -left-1 w-2.5 h-2.5 bg-blue-400 border border-white" />
              <div className="absolute -bottom-1 -right-1 w-2.5 h-2.5 bg-blue-400 border border-white" />
            </>
          )}
        </div>
      )}

      {/* Action buttons — ABOVE the box-shadow */}
      {confirmed && rect && (
        <div
          className="absolute flex gap-2"
          style={{
            left: rect.x,
            top: rect.y + rect.height + 12,
            zIndex: 50,
          }}
        >
          <button
            onClick={handleConfirm}
            className="px-4 py-1.5 bg-blue-500 hover:bg-blue-400 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
          >
            Valider
          </button>
          <button
            onClick={handleReset}
            className="px-4 py-1.5 bg-zinc-700 hover:bg-zinc-600 text-white text-sm font-medium rounded-lg transition-colors cursor-pointer"
          >
            Redessiner
          </button>
          <button
            onClick={closeOverlay}
            className="px-4 py-1.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 text-sm font-medium rounded-lg transition-colors cursor-pointer"
          >
            Annuler
          </button>
        </div>
      )}
    </div>
  );
}
