# Professional Photo Editing Engine

Keepix includes a custom, high-fidelity photo editing pipeline. Unlike typical image viewers that rely on basic CSS filters (which do not export and suffer from rendering artifacts), Keepix utilizes an in-browser **HTML5 Canvas Pixel-Level Rendering Engine** (`image-editor.ts`) coupled with a database persistence model.

---

## 🎨 Engine Overview

The editing engine processes raw image arrays in real time. It reads the source image into an offscreen canvas, extracts the `ImageData` pixel array (RGBA), and applies linear transformation matrices for exposure, temperature, and tint, along with custom convolutions for sharpening and blurring.

This method allows **non-destructive editing**:
1. The original source file on your SSD remains untouched.
2. All adjustments (slider values, presets, cropping dimensions, healing points) are stored as a JSON payload in the SQLite database (`media_items.adjustments`).
3. Svelte renders the adjusted canvas dynamically.
4. When exporting, Keepix processes the source file through the engine, outputting a fully baked JPEG/PNG incorporating all adjustments.

---

## 🎛️ Supported Adjustments

| Adjustment | Range | Algorithm / Details |
|---|---|---|
| **Exposure** | `-100` to `+100` | Adjusts RGB values linearly. Positive increases highlight levels; negative decreases low-light levels. |
| **Contrast** | `-100` to `+100` | Scales values around the midpoint grey (`128`). Expands (high contrast) or contracts (low contrast) color range. |
| **Highlights** | `-100` to `+100` | Selectively targets pixels with luminance $> 170$, adjusting their values without crushing shadows. |
| **Shadows** | `-100` to `+100` | Targets pixels with luminance $< 85$, lifting detail or deepening shadows. |
| **Temperature** | `-100` to `+100` | Shifts the balance between amber/orange (warmth) and blue (coolness). |
| **Tint** | `-100` to `+100` | Shifts the balance between magenta and green. |
| **Saturation** | `-100` to `+100` | Boosts or desaturates all color channels uniformly. |
| **Vibrance** | `-100` to `+100` | Smart saturation. Boosts less saturated colors while leaving skin tones and already saturated colors untouched. |
| **Clarity** | `-100` to `+100` | Increases midtone local contrast using a fast unsharp-mask filter. |
| **Vignette** | `0` to `100` | Draws a radial gradient mask that darkens the outer corners of the image. |
| **Sharpening** | `0` to `100` | Applies a $3 \times 3$ laplacian kernel convolution to highlight edge details. |
| **Noise Reduction** | `0` to `100` | Applies a selective bilateral smoothing filter to remove chroma and luma noise. |

---

## ⚡ Professional Presets

Keepix features 13 built-in presets designed for quick stylization. Presets set standard coordinates across all sliders:

1. **Original** (`original`): Resets all adjustments to `0`.
2. **Auto-Correction** (`auto`): Balances histogram exposure, bumping contrast and shadows slightly.
3. **High-Contrast B&W** (`bw`): Strips color (saturation `-100`) and pumps contrast (`+25`), exposure (`+5`), and shadows (`-10`).
4. **Vintage Film** (`vintage`): Increases warmth (`+15`), adds tint (`+10`), fades shadows (`+15`), and pulls contrast (`-10`).
5. **Cinematic Cool** (`cool_matte`): Shifts temperature to blue (`-20`), lifts shadows (`+12`), and desaturates slightly (`-10`).
6. **Warm Vibe** (`warm_vibe`): Temperature (`+25`), exposure (`+5`), contrast (`+8`).
7. **High Key** (`high_key`): Brightens exposure (`+30`), reduces contrast (`-15`), softens shadows (`+25`).
8. **Dramatic Grit** (`dramatic`): Contrast (`+35`), saturation (`-30`), shadows (`-15`), clarity (`+25`).
9. **Cyberpunk** (`cyberpunk`): Temperature (`-40`), tint (`+40`), exposure (`+8`), contrast (`+15`).
10. **Emerald Coast** (`emerald`): Shifts yellow/green channels, temperature (`-10`), tint (`-20`), saturation (`+15`).
11. **Soft Pastel** (`pastel`): Exposure (`+15`), saturation (`-15`), contrast (`-10`), shadows (`+30`).
12. **Cinematic Noir** (`noir`): High-contrast monochrome, shadows (`-25`), contrast (`+40`), vignette (`+35`).
13. **Vibrant Velvia** (`velvia`): High saturation (`+35`), contrast (`+12`), exposure (`+2`).

---

## 🔀 Before / After Comparison

To review edits, Keepix provides an interactive **Before/After Comparison** panel (triggered by hotkey `B`):
- **Draggable Split Slider**: Divides the screen into two halves. Sliding left/right dynamically shifts the mask boundary, rendering the original image on the left and the edited canvas on the right.
- **Side-by-Side View**: Renders the original and edited media side-by-side in separate viewport frames, with synchronized panning and zooming.

---

## 🩹 Spot Healing Tool

The spot healing tool provides automatic blemish removal:
- **Interactive Brush**: Use the mouse wheel to adjust the brush radius.
- **Content-Aware Fill**: When you click on a blemish, Keepix samples surrounding pixels outside the brush circle. It calculates a smooth gradient mask and interpolates textures into the center, effectively removing dust spots, skin blemishes, or small stray objects.
- **Non-Destructive Points**: Healing actions are recorded as coordinate points `(x, y, radius)` inside the adjustments JSON. They can be edited, toggled, or cleared at any time.

---

## 📋 Copying & Pasting Adjustments

If you are culling a sequence shot under the same lighting:
1. Adjust the first image to perfection.
2. Press `Ctrl + Shift + C` (or `Cmd + Shift + C` on macOS) to copy the entire adjustments schema to the clipboard.
3. Navigate to the next image and press `Ctrl + Shift + V`.
4. The canvas instantly applies the presets, sliders, and spot healing patches in real time.
