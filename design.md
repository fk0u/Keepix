# Keepix Architecture & Design Document

Keepix is a professional-grade, desktop-native, zero-latency photo and video culling workstation. This document details the technical design, architectural choices, rendering pipelines, and specific patterns implemented to make Keepix the *Expert and Creator Choice*.

---

## 1. Architectural Overview

Keepix is built on a hybrid architecture that leverages Svelte 5 for rendering reactive, high-performance UI and Rust for system-level speed, SIMD image decoding, and metadata extraction.

```
┌──────────────────────────────────────────────────────────┐
│                   SvelteKit 5 Frontend                   │
│      (Reactive Runes, Virtual Grid, Offscreen Canvas)    │
└────────────────────────────┬─────────────────────────────┘
                             │ (JSON IPC / Tauri Custom Asset Protocol)
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
│ │  - Custom binary XMP / Lightroom Preset extractor    │ │
│ └──────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

### Key Technical Pillars
- **Local-First & In-Memory Previews**: All media stays local. Thumbnails and previews are generated on-the-fly and cached in an optimized SQLite database with Write-Ahead Logging (WAL).
- **SIMD Acceleration**: Image resizing utilizes vector instructions (AVX2/SSE4.1/NEON) through the `fast_image_resize` crate.
- **Svelte 5 Runes**: The UI utilizes Svelte 5's fine-grained reactivity system (`$state`, `$derived`, `$effect`) to eliminate unnecessary DOM rerenders and maintain 60 FPS scrolling.

---

## 2. Photo Editor Engine & Pipeline

The Keepix editing pipeline is non-destructive and operates on pixel buffers inside the browser canvas element.

```
Original Image (WebP Cache) 
          │
          ▼
Loaded into HTMLImageElement
          │
          ▼
Rotation & Flip Matrix applied (ctx.transform)
          │
          ▼
ImageData extracted (ctx.getImageData)
          │
          ▼
applyAdjustments (Per-pixel pass: Exposure, Contrast, HSL Temperature/Tint)
          │
          ▼
applyClarity (Difference-of-Gaussians Box Blur)
          │
          ▼
applySharpen (Laplacian 3x3 Convolution Kernel)
          │
          ▼
Vignette & Film Grain noise applied
          │
          ▼
Written back to Canvas (ctx.putImageData)
```

---

## 3. Lightroom Preset Parser

Keepix imports Lightroom develop settings from `.xmp`, `.lrtemplate`, or RAW photo formats (which have embedded XMP namespaces like `<x:xmpmeta>`).

### Metadata Extraction (Rust Backend)
The `extract_xmp_preset` command parses binary files or sidecars for XMP strings. For RAW formats, it performs a fast sequential scanner seeking the XMP payload:
- Scans files up to 25MB for `<x:xmpmeta` and `</x:xmpmeta>` blocks.
- Returns the XML string to Svelte.

### Normalization & Mapping (Frontend)
Lightroom and Keepix use different adjustment ranges. Svelte's `parseLightroomPreset` maps these ranges:
- **Exposure**: Lightroom uses `-5.0` to `5.0` EV offsets. Keepix maps this linearly to `-100` to `100` via `Math.round(exp * 20)`.
- **Highlights & Shadows**: Maps the `Highlights2012` and `Shadows2012` attributes directly.
- **Temperature**: Maps Kelvin values (e.g. `5500` is neutral) to a relative `-100` to `100` slider.

---

## 4. Multi-Canvas Grid Rendering

In **2-Up** and **4-Up** compare views, creators must see all images with their respective adjustments applied.

### State Design
Instead of holding a single canvas or image reference, `PreviewView.svelte` manages records of loaded assets:
- `editCanvases`: `Record<string, HTMLCanvasElement>` (Canvases indexed by media ID).
- `sourceImages`: `Record<string, HTMLImageElement>` (Raw loaded source images).
- `originalImageSrcs`: `Record<string, string>` (Raw URL references).

### Synchronization Effect
A single reactive `$effect` monitors the list of `activeItems` being compared:
1. Prunes references to any media item no longer present in the active comparison grid.
2. Checks if the source image is already cached. If yes, it renders the canvas synchronously, preventing flashing.
3. If not cached, it loads the image asynchronously and draws it to the canvas.

---

## 5. UI Layout Stability & Preset Previews

A key challenge when introducing preset hover previews and active adjustment lists is preventing **Layout Shift loops**.

### The Layout Shift Loop Problem
1. User hovers their mouse over a preset button.
2. Svelte applies the preset adjustments temporarily to show a preview.
3. The derived list of active adjustments updates, and the "Active Adjustments HUD" expands, shifting the preset buttons down.
4. The preset button moves away from the mouse cursor, firing `onmouseleave` (restore state).
5. The HUD collapses, shifting the buttons back under the cursor, re-triggering hover.
6. This causes rapid flickering, rendering preset selection unusable.

### The Solution: Hover State Isolation
Keepix solves this by freezing the HUD's active adjustments display during hover. Svelte's derived state only calculates edits based on the *committed* adjustments:

```typescript
let activeEditsList = $derived.by(() => {
  const list: { name: string; value: string }[] = [];
  // If hovering, freeze the HUD content to the committed state
  const targetAdjustments = isHoveringPreset && originalAdjustmentsBeforeHover 
    ? originalAdjustmentsBeforeHover 
    : adjustments;

  for (const key of Object.keys(DEFAULT_ADJUSTMENTS)) {
    if (targetAdjustments[key] !== DEFAULT_ADJUSTMENTS[key]) {
      // Add to HUD display...
    }
  }
  return list;
});
```

Because `targetAdjustments` stays locked to the committed adjustments during the hover, the HUD container height never shifts, giving a smooth and professional hover-preview experience.

---

## 6. Sidebar Collapse Curtain Transitions

The Edit Panel minimization utilizes a curtain animation system:
- Rather than unmounting, the edit panel sets `width: 0px` and `min-width: 0px` with `overflow: hidden`.
- Contents are wrapped in a nested wrapper with `width: [expandedSize]px` and `flex-shrink: 0`. This keeps the child controls in place during transitions instead of squeezing them together.
- Transition durations are deactivated dynamically when the user is dragging split handles (`isDragging`), ensuring near-zero dragging stutters.
