import { useState, useRef } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";

interface Props {
  filePath: string;
  onClose: () => void;
}

export function VideoPreview({ filePath, onClose }: Props) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const [error, setError] = useState(false);

  const videoSrc = convertFileSrc(filePath);

  const extractFilename = (path: string) => {
    const parts = path.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || path;
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm animate-fade-in">
      <div className="relative bg-zinc-900 rounded-2xl shadow-2xl overflow-hidden max-w-[85vw] max-h-[85vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-zinc-800">
          <span className="text-sm text-zinc-300 font-medium truncate max-w-[400px]">
            {extractFilename(filePath)}
          </span>
          <button
            onClick={onClose}
            className="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-zinc-800 text-zinc-400 hover:text-white transition-colors"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        {/* Video */}
        <div className="flex-1 flex items-center justify-center bg-black min-h-[300px]">
          {error ? (
            <div className="text-zinc-500 text-sm text-center p-8">
              <svg className="mx-auto mb-3 text-zinc-600" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="10" />
                <line x1="15" y1="9" x2="9" y2="15" />
                <line x1="9" y1="9" x2="15" y2="15" />
              </svg>
              Impossible de lire la vidéo
            </div>
          ) : (
            <video
              ref={videoRef}
              src={videoSrc}
              controls
              autoPlay
              className="max-w-full max-h-[70vh] outline-none"
              onError={() => setError(true)}
            />
          )}
        </div>
      </div>
    </div>
  );
}
