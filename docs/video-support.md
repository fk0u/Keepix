# Video Playback & Culling System

Keepix isn't just for photo sorting. It features a complete **High-Performance Video Culling Engine** that handles professional video containers and codecs (including `.mp4`, `.mov`, and `.mkv`) with zero-latency playback and EXIF metadata scraping.

---

## 🎥 Supported Formats & Decoders

Keepix handles modern video codecs by utilizing the host operating system's native hardware-accelerated media frameworks via the Tauri webview wrapper.

- **Containers Supported**: `.mp4`, `.mov`, `.mkv`
- **Codecs Supported**:
  - **H.264 (AVC)**: Fully supported across all systems.
  - **H.265 (HEVC)**: Supported natively on macOS and Windows 10/11 (requires *HEVC Video Extensions* from the Microsoft Store on some Windows variants).
  - **ProRes / RAW**: Best culled by opening in an external player via the system link, or pre-rendered proxy workflows.

---

## 🛠️ Video Player Features

The embedded player (`VideoPlayer.svelte`) features a custom-built, keyboard-controllable interface:
- **Play/Pause**: Press `K` or `Space` (when video is focused) to play/pause.
- **Scrubbing**: Click-and-drag the timeline track or use `Arrow Left/Right` to skip frame-by-frame.
- **Loop Mode**: Toggle repeating playbacks automatically.
- **Audio Control**: Mute/unmute and slider volume controls are saved persistently across culling sessions.
- **Speed Controller**: Speed up video playback (up to `2x`) to cull long footage in half the time.

---

## ⚡ Video Processing Backend

When the multi-threaded scanner (`scanner.rs`) detects a video file:
1. **Header Parsing**: The backend opens the video file's metadata descriptors to extract duration, video resolution, frames-per-second, and compression codec.
2. **Database Registration**: The details are saved to the database via `save_video_metadata`.
3. **System Integration**:
   - If a video container fails to load (due to missing codecs on the client OS), Svelte captures the `onerror` event.
   - It fires `commands::mark_video_unsupported`, writing an error flag to the SQLite entry.
   - The UI replaces the black screen with an informative warning, suggesting the installation of codec packs or providing a **"Open in External Player"** link which leverages Rust's `open` command to launch the file in VLC, QuickTime, or Windows Media Player.
