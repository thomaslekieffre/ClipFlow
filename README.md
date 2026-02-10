# ClipFlow

Screen recording & assembly tool for Windows. Record screen regions, arrange clips on a timeline, add transitions between them, preview the result, and export to MP4.

Built with **Tauri v2** (Rust) + **React 19** + **FFmpeg**.

![Tauri](https://img.shields.io/badge/Tauri-v2-blue?logo=tauri)
![React](https://img.shields.io/badge/React-19-61dafb?logo=react)
![FFmpeg](https://img.shields.io/badge/FFmpeg-sidecar-green)
![License](https://img.shields.io/badge/license-MIT-yellow)

## Doc précise générée :
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/thomaslekieffre/ClipFlow)

## Features

- **Region capture** — Select any screen area or record fullscreen, multi-monitor and HiDPI support
- **Timeline** — Drag & drop clips to reorder them
- **19 transitions** — Fade, dissolve, slides, wipes, zoom, pixelize, iris, radial, smooth...
- **Video preview** — Fast low-quality render to check the result before exporting
- **MP4 export** — Full quality export with FFmpeg xfade transitions
- **Watermark** — Optional "ClipFlow" watermark on exported videos, toggleable
- **Global hotkeys** — `F9` to record/stop, `ESC` to cancel
- **Dark/Light theme** — Toggle with persistence
- **System notifications** — Toast notification when export completes

## Tech Stack

| Layer | Tech |
|-------|------|
| Desktop framework | Tauri v2 |
| Backend | Rust |
| Frontend | React 19, TypeScript, Vite 7 |
| Styling | TailwindCSS v4 |
| State | Zustand |
| DnD | @dnd-kit |
| Video | FFmpeg (auto-downloaded via ffmpeg-sidecar) |

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) >= 1.70
- Windows 10/11

### Install & Run

```bash
git clone https://github.com/thomaslekieffre/ClipFlow.git
cd ClipFlow
npm install
npx tauri dev
```

FFmpeg is downloaded automatically on first launch.

### Build for Production

```bash
npx tauri build
```

The installer will be in `src-tauri/target/release/bundle/`.

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `F9` | Start / Stop recording |
| `ESC` | Cancel recording |

## Project Structure

```
src/                    # React frontend
  components/
    controls/           # RecordButton, StatusIndicator, ThemeToggle
    timeline/           # Timeline, SortableClipCard, TransitionIcon
    export/             # ExportButton
    preview/            # VideoPreview
    overlay/            # RegionOverlay
  stores/               # Zustand store
  lib/                  # Types, Tauri API wrapper

src-tauri/              # Rust backend
  src/
    capture/            # FFmpeg screen capture (gdigrab)
    recording/          # Recording state machine
    region/             # Region selection logic
    export/             # FFmpeg export with xfade transitions
    hotkeys.rs          # Global shortcuts (F9, ESC)
    commands.rs         # Tauri IPC commands
    types.rs            # Shared types
    state.rs            # App state
```

## Author

**Thomas Lekieffre** — [GitHub](https://github.com/thomaslekieffre)

## License

MIT
