# Getting Started with Keepix

Welcome to **Keepix**, the ultimate high-performance photo and video culling workstation designed for professional content creators, wedding photographers, and production studios. 

Keepix was built to eliminate the latency bottleneck in creative workflows. Instead of waiting for heavy editing software to load raw assets, Keepix enables you to scan, sort, cull, and refine thousands of files in real-time, utilizing keyboard-first interactions.

---

## ⚡ Quick Start Guide

### Step 1: Create or Open a Project
Upon launching Keepix, you will be greeted by the **Home Screen**. 
- Click **"New Project"** (or press `Ctrl + N` / `Cmd + N`).
- Give your project a descriptive name (e.g., `Wedding Session - June 2026`).
- Select the **Root Directory** containing your camera media. Keepix scans recursively to discover photos and videos.
- Click **"Create Project"**.

### Step 2: The Culling Workspace
Once loaded, you will enter the main workspace:
1. **Media Grid (Left Pane)**: A virtualized grid showcasing automatically generated, high-resolution thumbnails.
2. **Preview Panel (Center Pane)**: Pixel-perfect preview rendering. Switch between a lightweight cached preview and a raw edit canvas.
3. **Control Center (Right Pane)**: Access metadata details (EXIF) and fine-grained image/video adjustment parameters.

### Step 3: Fast Keyboard Culling
The core Keepix philosophy is to keep your hands on the keyboard. Use these keybindings to sort your media:
- `Arrow Left / Right`: Navigate between media items with sub-millisecond selection latency.
- `Space`: Toggle between full-screen preview and the media grid.
- **Categorization Hotkeys**:
  - `1` : **Trash / Discard** (Red label) — Media flagged for permanent deletion.
  - `2` : **Keep / Best Shots** (Green label) — High-quality selects.
  - `3` : **Draft / Backup** (Blue label) — Secondary items.
  - `4` : **Review Needed** (Yellow label) — Requires further analysis.
- `Ctrl + Z` / `Cmd + Z`: Instantly undo your last sorting classification.

### Step 4: Batch Export
When your culling session is complete:
- Click **Export** in the top navigation bar.
- Choose your criteria (e.g., export only `Category 2: Keep` items).
- Define your destination folder and file naming pattern.
- Keepix will copy the structured files in a fast background worker.

---

## ⌨️ Advanced Keyboard Shortcuts

To operate Keepix like an expert, master these system hotkeys:

### Navigation & Views
| Shortcut | Action | Description |
|---|---|---|
| `Arrow Left / Right` | Move Selection | Navigate to the previous/next item. |
| `Space` | Toggle Preview | Expand selected media to full size or return to grid. |
| `I` | Toggle EXIF Info | Open or collapse the side metadata inspector panel. |
| `?` | Keyboard Help | Overlay the shortcut reference guide. |

### Sorting & Flagging
| Shortcut | Action | Description |
|---|---|---|
| `1` | Discard (Trash) | Mark media as category 1. |
| `2` | Keep (Best) | Mark media as category 2. |
| `3` | Draft / Secondary | Mark media as category 3. |
| `4` | Review | Mark media as category 4. |
| `Delete` | Quick Discard | Instantly flag current item as Trash. |
| `Ctrl + Z` | Undo | Revert the last sorting action (unlimited history). |

### Professional Editing
| Shortcut | Action | Description |
|---|---|---|
| `B` | Toggle Before/After | Instant preview of editing adjustments vs original. |
| `Ctrl + Shift + C` | Copy Adjustments | Copy all sliders, healing patches, and presets. |
| `Ctrl + Shift + V` | Paste Adjustments | Apply copied adjustments to the current media. |

---

## ⚙️ Minimum System Requirements

Keepix is highly optimized, but these specs are recommended for smooth 4K video playback and RAW image parsing:
- **OS**: Windows 10/11 (64-bit), macOS 12+, Linux (Ubuntu 20.04+)
- **CPU**: Intel Core i5 / AMD Ryzen 5 (Multicore with SIMD/AVX support highly recommended)
- **RAM**: 8 GB RAM minimum (16 GB recommended for culling 4K workflows)
- **Storage**: SSD (NVMe is preferred for lightning-fast disk reads)
- **Display**: 1920x1080 resolution or higher
