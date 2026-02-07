import type { RecordingState } from "../../lib/types";

interface Props {
  state: RecordingState;
  onClick: () => void;
  disabled?: boolean;
}

export function RecordButton({ state, onClick, disabled }: Props) {
  const isRecording = state === "recording";
  const isPaused = state === "paused";

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className={`
        flex items-center gap-2 px-5 py-2.5 rounded-lg text-sm font-semibold transition-all
        ${
          isRecording || isPaused
            ? "bg-zinc-200 text-zinc-900 hover:bg-white"
            : "bg-red-600 text-white hover:bg-red-500"
        }
        ${disabled ? "opacity-40 cursor-not-allowed" : "cursor-pointer"}
      `}
    >
      {isRecording ? (
        <>
          <span className="w-3 h-3 rounded-sm bg-zinc-900" />
          Stop
        </>
      ) : isPaused ? (
        <>
          <span className="w-3 h-3 rounded-sm bg-zinc-900" />
          Stop
        </>
      ) : (
        <>
          <span className="w-3 h-3 rounded-full bg-white" />
          Record
        </>
      )}
    </button>
  );
}
