<div align="center">
 
# 📷 Keepix
### *The Zero-Latency, Local-First Photo & Video Culling Workstation for Professional Creators*
 
[![Tauri v2](https://img.shields.io/badge/Tauri-v2.0-blue?style=for-the-badge&logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![SvelteKit v5](https://img.shields.io/badge/SvelteKit-v5.0-ff3e00?style=for-the-badge&logo=svelte&logoColor=white)](https://svelte.dev/)
[![Rust Core](https://img.shields.io/badge/Rust-Backend-dea584?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Database SQLite](https://img.shields.io/badge/Database-SQLite_WAL-003b57?style=for-the-badge&logo=sqlite&logoColor=white)](https://sqlite.org/)
[![Local First](https://img.shields.io/badge/Privacy-100%25_Local--First-success?style=for-the-badge)](https://localfirst.fm/)
 
<p align="center">
  <strong>Keepix</strong> is a blazing-fast, desktop-native photo and video culling app designed specifically for professional photographers, videographers, and post-production studios. By offloading resource-heavy tasks to multi-threaded Rust binaries and SIMD-accelerated processors, Keepix delivers near-zero latency, allowing you to sort, inspect, and draft tens of thousands of media items in record time.
</p>
 
[✨ Getting Started](docs/getting-started.md) • [🏗️ Technical Architecture](docs/architecture.md) • [🎨 Photo Editor Engine](docs/photo-editing.md) • [🎥 Video Culling](docs/video-support.md) • [🔑 Installer & Signing](docs/installer-guide.md) • [🌐 Localization](docs/internationalization.md)
 
</div>
 
---
 
## 🚀 Why Keepix?
 
For professional creators, the first stage of post-production—culling—is often the biggest bottleneck. Heavy creative suites are built for editing, not sorting; they suffer from high image loading overhead and UI stuttering. 
 
Keepix solves this by treating culling as a performance-critical system operation:
*   **Sub-millisecond navigation**: Move between RAW and compressed image assets instantly.
*   **Zero cloud reliance**: Your media never leaves your SSD. Privacy is built-in.
*   **System-level acceleration**: Resizes thumbnails in background threads using raw CPU SIMD instructions.
*   **Persistent undo chain**: Maintain unlimited history of sorting categories across app restarts.
 
---
 
## 🌟 Key Features
 
### ⚡ Lightning-Fast Media Scanning & Scaling
- **Concurrently Scanned Catalogs**: The backend scanner indexes directories recursively on background threads, processing 10,000+ images in seconds.
- **SIMD Hardware Resizing**: Leverages `fast_image_resize` using AVX2, SSE4, or ARM NEON vector instructions to output highly compressed WebP previews.
- **Preloading Worker**: Smart look-ahead cache decodes the next 2 images in your selection sequence before you click, eliminating loading screens.
 
### ⌨️ Keyboard-First Culling Interface
- **4-Category Sorting Scheme**: Instantly tag media into customizable workflows:
  - `1` : **Trash / Discard** (Red) — Mark files for permanent deletion.
  - `2` : **Keep / Best Shots** (Green) — Master selection.
  - `3` : **Draft / Secondary** (Blue) — Low-priority keeps.
  - `4` : **Review Needed** (Yellow) — Requires closer examination.
- **Auto-Advance Mode**: Once categorized, Keepix advances your selection to the next item automatically, letting you cull at the speed of thought.
- **Full History**: Press `Ctrl + Z` to undo sorting mistakes, backed by database transaction rollbacks.
 
### 🎨 Non-Destructive Canvas Photo Editor (v4.1.0 Upgrades)
- **HTML5 Canvas Pipeline**: Pixel-level adjustments applied instantly without layout reflows or CPU bottlenecks.
- **12 Sliders**: Control Exposure, Contrast, Highlights, Shadows, Temperature, Tint, Saturation, Vibrance, Clarity, Vignette, Sharpening, and Noise Reduction.
- **Live Preset Hover Previews**: Hover over presets (built-in or custom) to instantly render the preset look in real-time. Unhovering immediately restores active editing state.
- **Custom Preset Manager**: Save active slider edits under a custom name, persisted in `localStorage`. 
- **Lightroom Preset Importer**: Import develop settings from `.xmp`, `.lrtemplate`, or RAW files (extracts embedded develop metadata) and save them to your Custom Presets.
- **Active Adjustments HUD**: High-utility HUD panel summarizing active edits with a single-click "Reset All" option, engineered with layout-stabilization to prevent preset-selection jumps.
 
### 🩹 Spot Healing Tool
- **Circular Healing Brush**: Adjust brush radius with mouse scroll.
- **Content-Aware Texture Synthesis**: Samples surrounding pixel gradients to automatically remove sensor dust, lens spots, or minor skin blemishes non-destructively.
 
### 🔀 Before/After Slider & Compare View
- **Interactive Split Slider**: A glassmorphic divider bar you can drag to inspect your adjustments vs the original raw pixel details.
- **Multi-Canvas Grid Rendering**: In **2-Up** and **4-Up** modes, every photo in the compare grid renders its respective adjustments in real-time, allowing side-by-side preset comparisons.
 
### 📼 Hardware-Accelerated Video Culling
- **Containers**: Plays `.mp4`, `.mov`, and `.mkv` files natively.
- **OS Native Integration**: Decodes 4K footage using GPU hardware encoders. Includes playback speed toggles (up to `2x`) for fast timeline scrubs.
- **Fail-Safe Fallbacks**: Auto-detects codec incompatibilities, displays diagnostic warnings, and provides links to launch unsupported items directly inside VLC or QuickTime.
 
---
 
## 🏗️ Technical Stack & Architecture
 
Keepix uses a custom hybrid architecture to combine Svelte's reactive bindings with Rust's bare-metal speed:
 
```
┌──────────────────────────────────────────────────────────┐
│                   SvelteKit 5 Frontend                   │
│      (Reactive Runes, Virtual Grid, Offscreen Canvas)    │
└────────────────────────────┬─────────────────────────────┘
                             │ (JSON IPC Messages / Asset Protocol)
┌────────────────────────────▼─────────────────────────────┐
│                    Tauri 2.0 Bridge                      │
│        (Command Dispatcher, Custom Asset Handlers)       │
└────────────────────────────┬─────────────────────────────┘
                             │
┌────────────────────────────▼─────────────────────────────┐
│                      Rust Backend                        │
│ ┌──────────────────────────────────────────────────────┐ │
│ │  - sqlite3 Database (WAL mode, index optimizations)  │ │
│ │  - SIMD fast_image_resize (AVX2 / SSE4.1 / NEON)    │ │
│ │  - kamadak-exif parser & metadata serializer         │ │
│ │  - In-Memory preloading LRU image cache              │ │
│ └──────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```
 
> [!NOTE]
> Read the complete architectural overview in our [Technical Architecture Guide](docs/architecture.md) to understand database schemes, concurrency layers, and SIMD optimizations.
 
---
 
## ⌨️ Workspace Shortcuts
 
| Category | Command | Hotkey |
| :--- | :--- | :--- |
| **Navigation** | Move Selection | `Arrow Left` / `Arrow Right` |
| | Toggle View Mode (Grid / Preview) | `Space` |
| | Toggle Metadata Panel | `i` |
| | Open Shortcut Reference | `?` |
| **Culling** | Mark Category 1 (Trash) | `1` |
| | Mark Category 2 (Keep) | `2` |
| | Mark Category 3 (Draft) | `3` |
| | Mark Category 4 (Review) | `4` |
| | Quick Trash | `Delete` |
| | Revert Last Action | `Ctrl + Z` |
| **Editing** | Toggle Before/After | `B` |
| | Copy Image Adjustments | `Ctrl + Shift + C` |
| | Paste Image Adjustments | `Ctrl + Shift + V` |
 
---
 
## 🛠️ Local Development & Setup
 
### Prerequisites
1. **Node.js** (v18.x or higher)
2. **Rust & Cargo** (v1.75 or higher)
3. **C++ Build Tools** (For Windows compilation, see the [Tauri Installation Guide](https://v2.tauri.app/start/prerequisites/))
 
### Development Flow
1. Clone the project repository and install dependencies:
   ```bash
   npm install
   ```
2. Launch the application in hot-reloading development mode:
   ```bash
   npm run tauri dev
   ```
   *The Svelte frontend compiles, the Rust backend starts, and Keepix opens in a native window.*
 
---
 
## 📦 Release Compilation
 
To build an optimized standalone executable installer:
 
```bash
npm run tauri build
```
 
This compiles Svelte assets, triggers compiler optimizations in the Rust backend (`-C target-cpu=native`), and generates:
- **Windows**: Branded NSIS installers (`.exe`) and Enterprise MSI packages (`.msi`).
- **macOS**: Signed and notarized DMG disk images (`.dmg`).
 
> [!TIP]
> To configure custom icons, NSIS installer layouts, background sidebars, or code signing certificates, refer to the [Custom Installer & Signing Guide](docs/installer-guide.md).
 
---
 
## 🌐 Localization (i18n)
 
Keepix is fully localization-ready, featuring dynamic locale switching between **English** and **Indonesian**:
- Translation configurations are stored as optimized key-value hashes in `src/lib/i18n/index.ts`.
- Language configurations are persisted locally in SQLite settings.
- Read the [Internationalization Guide](docs/internationalization.md) to learn how to add custom locale mappings.
 
---
 
## 📄 License
 
Keepix is open-source software licensed under the [MIT License](LICENSE).
