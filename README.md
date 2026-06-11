# Keepix

**Keepix** is a fast, lightweight, and powerful photo & video culling application built for professional photographers and videographers. It is designed to help users quickly sort thousands of photos/videos into 4 customizable categories using a keyboard-first, highly efficient workflow.

## Features

- **Blazing Fast Performance**: Built with Tauri 2 and Rust, utilizing background multi-threading, SQLite, and SIMD-accelerated image resizing (`fast_image_resize`) to handle 10,000+ files effortlessly.
- **Keyboard-First Workflow**: Never take your hands off the keyboard. Sort images at the speed of thought.
- **Hotkeys**:
  - `1` → Buang (Trash/Discard)
  - `2` → Simpan (Best Shots)
  - `3` → Simpan Draft (Low priority keep)
  - `4` → Review (Needs further check)
  - `Arrow Left/Right` → Navigate between media
  - `Space` → Toggle full-size Preview / Grid view
  - `Delete` → Quick move to Buang
  - `Ctrl + Z` / `Cmd + Z` → Undo last action
  - `i` → Toggle EXIF Metadata Panel
  - `?` → Show Keyboard Shortcuts
- **Instant Preview & Grid View**: Switch seamlessly between a high-performance virtualized grid view and a full-size preview.
- **EXIF Metadata Support**: Automatically extracts and displays camera make, lens model, exposure settings (ISO, Aperture, Shutter Speed), and more.
- **Fully Local & Offline**: No cloud uploads, no telemetry. Keepix respects your privacy and works completely offline.
- **Modern Premium UI**: Sleek, dark-mode, glassmorphic UI built with SvelteKit.

## Tech Stack

- **Frontend**: SvelteKit 5 (SPA Mode), TypeScript, Vanilla CSS (Glassmorphism design system)
- **Backend**: Rust, Tauri 2.0
- **Database**: SQLite (`rusqlite`) for fast querying, persistent states, and undo history
- **Image Processing**: `image` and `fast_image_resize` (SIMD) for high-speed thumbnail generation
- **Metadata**: `kamadak-exif` for robust EXIF parsing

## Prerequisites

Before you begin, ensure you have the following installed:

1. [Node.js](https://nodejs.org/) (v18 or newer)
2. [Rust](https://www.rust-lang.org/tools/install)
3. Windows build dependencies (C++ Build Tools) if you're on Windows. (Follow the [Tauri setup guide](https://v2.tauri.app/start/prerequisites/)).

## How to Run in Development Mode

To start the application in development mode with hot-reloading for both the Svelte frontend and the Rust backend:

1. Open your terminal and navigate to the project directory:
   ```bash
   cd d:/Project/Keepix
   ```
2. Install the Node.js dependencies:
   ```bash
   npm install
   ```
3. Start the Tauri development server:
   ```bash
   npm run tauri dev
   ```
   *Note: The first time you run this command, it might take a few minutes for Rust to download and compile all the backend dependencies. Subsequent runs will be much faster.*

## How to Build for Production

To create a standalone executable for your operating system:

```bash
npm run tauri build
```
Once the build is complete, you can find the installer/executable inside the `src-tauri/target/release/bundle/` directory.

## App Architecture overview

- **`src/`**: Contains the SvelteKit frontend UI.
  - `lib/components/`: Reusable UI elements (GridView, PreviewView, Sidebar, etc.)
  - `lib/services/`: Services bridging the frontend with the Rust backend (`tauri-bridge.ts`).
  - `lib/stores/`: Svelte 5 `$state` stores for global application state (media, project, UI).
  - `routes/cull/`: The main culling workspace view.
- **`src-tauri/src/`**: Contains the Rust backend.
  - `main.rs`: Entry point for the Tauri application.
  - `commands.rs`: IPC commands invoked by the frontend.
  - `db.rs`: SQLite database schema and CRUD operations.
  - `scanner.rs`: Background folder scanning and file discovery.
  - `thumbnail.rs`: High-performance SIMD thumbnail generation.
  - `metadata.rs`: EXIF metadata extraction.
- **`Keepix Database`**: Keepix creates a local SQLite database (`keepix.db`) inside your OS's `AppData` / `Application Support` directory to store project states, categories, and undo history.

## License

This project is open-source and available under the MIT License.
