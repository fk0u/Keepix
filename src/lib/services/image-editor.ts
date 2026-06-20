// ============================================================================
// Keepix — Image Editor Engine
// Pure TypeScript pixel-level image processing for non-destructive editing.
// All adjustments are applied to ImageData for real-time canvas rendering.
// ============================================================================

export interface Adjustments {
  exposure: number;      // -100 to 100
  contrast: number;      // -100 to 100
  highlights: number;    // -100 to 100
  shadows: number;       // -100 to 100
  temperature: number;   // -100 to 100
  tint: number;          // -100 to 100
  saturation: number;    // -100 to 100
  vibrance: number;      // -100 to 100
  sharpen: number;       // 0 to 100
  clarity: number;       // -100 to 100
  vignette: number;      // 0 to 100
  grain: number;         // 0 to 100
  rotation: number;      // 0, 90, 180, 270
  flipH: boolean;
  flipV: boolean;
}

export const DEFAULT_ADJUSTMENTS: Adjustments = {
  exposure: 0,
  contrast: 0,
  highlights: 0,
  shadows: 0,
  temperature: 0,
  tint: 0,
  saturation: 0,
  vibrance: 0,
  sharpen: 0,
  clarity: 0,
  vignette: 0,
  grain: 0,
  rotation: 0,
  flipH: false,
  flipV: false,
};

/** All available preset definitions */
export interface PresetDef {
  id: string;
  labelKey: string;
  values: Partial<Adjustments> | null;
}

export const PRESETS: PresetDef[] = [
  { id: 'none', labelKey: 'edit.preset.none', values: null },
  { id: 'cinematic', labelKey: 'edit.preset.cinematic', values: { exposure: -10, contrast: 20, highlights: -15, shadows: 10, temperature: -5, tint: 5, saturation: -15, vibrance: 10 } },
  { id: 'vintage', labelKey: 'edit.preset.vintage', values: { exposure: 10, contrast: -15, highlights: -20, shadows: 25, temperature: 15, tint: -5, saturation: -20, vibrance: -10 } },
  { id: 'bw', labelKey: 'edit.preset.bw', values: { exposure: 0, contrast: 30, highlights: 20, shadows: -20, temperature: 0, tint: 0, saturation: -100, vibrance: 0 } },
  { id: 'punchy', labelKey: 'edit.preset.punchy', values: { exposure: 5, contrast: 15, highlights: -10, shadows: 5, temperature: 0, tint: 0, saturation: 10, vibrance: 25 } },
  { id: 'fade', labelKey: 'edit.preset.fade', values: { exposure: 10, contrast: -25, highlights: -10, shadows: 30, temperature: 5, tint: 0, saturation: -10, vibrance: -5 } },
  { id: 'warm_sunset', labelKey: 'edit.preset.warm_sunset', values: { exposure: 5, contrast: 10, highlights: -5, shadows: 15, temperature: 35, tint: 10, saturation: 15, vibrance: 20 } },
  { id: 'cool_breeze', labelKey: 'edit.preset.cool_breeze', values: { exposure: 0, contrast: 5, highlights: 10, shadows: 0, temperature: -30, tint: -10, saturation: -5, vibrance: 10 } },
  { id: 'moody', labelKey: 'edit.preset.moody', values: { exposure: -15, contrast: 25, highlights: -25, shadows: -10, temperature: -10, tint: 5, saturation: -20, vibrance: 5, vignette: 40 } },
  { id: 'film_noir', labelKey: 'edit.preset.film_noir', values: { exposure: -5, contrast: 40, highlights: 15, shadows: -30, temperature: 0, tint: 0, saturation: -100, vibrance: 0, vignette: 50, grain: 15 } },
  { id: 'pastel', labelKey: 'edit.preset.pastel', values: { exposure: 15, contrast: -20, highlights: 10, shadows: 20, temperature: 5, tint: -5, saturation: -25, vibrance: 15 } },
  { id: 'hdr', labelKey: 'edit.preset.hdr', values: { exposure: 0, contrast: 20, highlights: -30, shadows: 40, temperature: 0, tint: 0, saturation: 15, vibrance: 30, clarity: 40 } },
  { id: 'sepia', labelKey: 'edit.preset.sepia', values: { exposure: 5, contrast: -5, highlights: -10, shadows: 10, temperature: 25, tint: 10, saturation: -60, vibrance: -20, vignette: 20 } },
];

// ============================================================================
// Core Pixel Processing
// ============================================================================

/** Clamp value between 0 and 255 */
function clamp(v: number): number {
  return v < 0 ? 0 : v > 255 ? 255 : v;
}

/** Convert RGB to HSL */
function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  let h = 0, s = 0;
  const l = (max + min) / 2;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case r: h = ((g - b) / d + (g < b ? 6 : 0)) / 6; break;
      case g: h = ((b - r) / d + 2) / 6; break;
      case b: h = ((r - g) / d + 4) / 6; break;
    }
  }
  return [h, s, l];
}

/** Convert HSL to RGB */
function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  if (s === 0) {
    const v = Math.round(l * 255);
    return [v, v, v];
  }
  const hue2rgb = (p: number, q: number, t: number) => {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1 / 6) return p + (q - p) * 6 * t;
    if (t < 1 / 2) return q;
    if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
    return p;
  };
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
  const p = 2 * l - q;
  return [
    Math.round(hue2rgb(p, q, h + 1 / 3) * 255),
    Math.round(hue2rgb(p, q, h) * 255),
    Math.round(hue2rgb(p, q, h - 1 / 3) * 255),
  ];
}

/**
 * Apply all adjustments to an ImageData buffer (in-place).
 * Performs a single pass over all pixels for maximum performance.
 */
export function applyAdjustments(
  srcData: ImageData,
  adj: Adjustments,
  width: number,
  height: number,
): ImageData {
  const data = new Uint8ClampedArray(srcData.data);
  const result = new ImageData(data, width, height);
  const len = data.length;

  // Pre-compute adjustment factors
  const exposureFactor = Math.pow(2, adj.exposure / 50); // -100→0.25x, 0→1x, 100→4x
  const contrastFactor = (adj.contrast + 100) / 100; // Maps -100→0, 0→1, 100→2
  const satFactor = 1 + adj.saturation / 100;
  const vibFactor = adj.vibrance / 100;
  const highlightsFactor = adj.highlights / 100;
  const shadowsFactor = adj.shadows / 100;
  const tempFactor = adj.temperature / 100;
  const tintFactor = adj.tint / 100;
  const grainAmount = adj.grain * 0.6; // Scale grain intensity
  const vignetteAmount = adj.vignette / 100;

  // Center coordinates for vignette
  const cx = width / 2;
  const cy = height / 2;
  const maxDist = Math.sqrt(cx * cx + cy * cy);

  // Simple PRNG for grain (deterministic per-frame)
  let seed = 12345;
  const pseudoRandom = () => {
    seed = (seed * 16807 + 0) % 2147483647;
    return (seed / 2147483647) * 2 - 1; // -1 to 1
  };

  for (let i = 0; i < len; i += 4) {
    let r = data[i];
    let g = data[i + 1];
    let b = data[i + 2];

    // 1. Exposure
    if (adj.exposure !== 0) {
      r = clamp(r * exposureFactor);
      g = clamp(g * exposureFactor);
      b = clamp(b * exposureFactor);
    }

    // 2. Highlights & Shadows (tonal range targeting)
    if (adj.highlights !== 0 || adj.shadows !== 0) {
      const luminance = (r * 0.299 + g * 0.587 + b * 0.114) / 255;
      
      if (luminance > 0.5 && adj.highlights !== 0) {
        // Affect highlights (bright areas)
        const blend = (luminance - 0.5) * 2; // 0→1 for highlights
        const shift = highlightsFactor * blend * 60;
        r = clamp(r + shift);
        g = clamp(g + shift);
        b = clamp(b + shift);
      }
      if (luminance < 0.5 && adj.shadows !== 0) {
        // Affect shadows (dark areas)
        const blend = (0.5 - luminance) * 2; // 0→1 for shadows
        const shift = shadowsFactor * blend * 60;
        r = clamp(r + shift);
        g = clamp(g + shift);
        b = clamp(b + shift);
      }
    }

    // 3. Contrast (S-curve around midtone)
    if (adj.contrast !== 0) {
      r = clamp(((r / 255 - 0.5) * contrastFactor + 0.5) * 255);
      g = clamp(((g / 255 - 0.5) * contrastFactor + 0.5) * 255);
      b = clamp(((b / 255 - 0.5) * contrastFactor + 0.5) * 255);
    }

    // 4. Temperature (warm ↔ cool shift)
    if (adj.temperature !== 0) {
      r = clamp(r + tempFactor * 30);
      b = clamp(b - tempFactor * 30);
    }

    // 5. Tint (green ↔ magenta shift)
    if (adj.tint !== 0) {
      g = clamp(g + tintFactor * 25);
      r = clamp(r - tintFactor * 10);
      b = clamp(b - tintFactor * 10);
    }

    // 6. Saturation & Vibrance (in HSL space)
    if (adj.saturation !== 0 || adj.vibrance !== 0) {
      const [h, s, l] = rgbToHsl(r, g, b);
      let newSat = s;

      // Saturation: uniform shift
      if (adj.saturation !== 0) {
        newSat = Math.max(0, Math.min(1, s * satFactor));
      }

      // Vibrance: boost low-saturation colors more
      if (adj.vibrance !== 0) {
        const vibranceBoost = vibFactor * (1 - s) * 0.5;
        newSat = Math.max(0, Math.min(1, newSat + vibranceBoost));
      }

      const [nr, ng, nb] = hslToRgb(h, newSat, l);
      r = nr; g = ng; b = nb;
    }

    // 7. Grain (film-style noise)
    if (adj.grain > 0) {
      const noise = pseudoRandom() * grainAmount;
      r = clamp(r + noise);
      g = clamp(g + noise);
      b = clamp(b + noise);
    }

    // 8. Vignette (darken edges)
    if (adj.vignette > 0) {
      const pixelIdx = i / 4;
      const px = pixelIdx % width;
      const py = Math.floor(pixelIdx / width);
      const dx = (px - cx) / cx;
      const dy = (py - cy) / cy;
      const dist = Math.sqrt(dx * dx + dy * dy);
      const vignetteFade = 1 - vignetteAmount * Math.pow(dist, 2) * 0.8;
      r = clamp(r * vignetteFade);
      g = clamp(g * vignetteFade);
      b = clamp(b * vignetteFade);
    }

    data[i] = r;
    data[i + 1] = g;
    data[i + 2] = b;
    // Alpha unchanged
  }

  return result;
}

/**
 * Apply a 3x3 unsharp mask (sharpen) convolution.
 * This is run as a separate pass because it requires neighboring pixel data.
 */
export function applySharpen(imageData: ImageData, amount: number): ImageData {
  if (amount <= 0) return imageData;
  
  const { width, height, data: src } = imageData;
  const result = new Uint8ClampedArray(src);
  const strength = amount / 100;

  for (let y = 1; y < height - 1; y++) {
    for (let x = 1; x < width - 1; x++) {
      const idx = (y * width + x) * 4;
      
      for (let c = 0; c < 3; c++) {
        // 3x3 Laplacian kernel for unsharp mask
        const center = src[idx + c] * 5;
        const neighbors = 
          src[((y - 1) * width + x) * 4 + c] +
          src[((y + 1) * width + x) * 4 + c] +
          src[(y * width + (x - 1)) * 4 + c] +
          src[(y * width + (x + 1)) * 4 + c];
        
        const sharpened = center - neighbors;
        result[idx + c] = clamp(src[idx + c] + sharpened * strength * 0.25);
      }
    }
  }

  return new ImageData(result, width, height);
}

/**
 * Apply clarity (local contrast enhancement) via a simplified approach.
 * Uses a difference-of-Gaussians-like method approximated by a simple box blur.
 */
export function applyClarity(imageData: ImageData, amount: number): ImageData {
  if (amount === 0) return imageData;
  
  const { width, height, data: src } = imageData;
  const result = new Uint8ClampedArray(src);
  const strength = amount / 200; // More subtle effect
  const radius = 3;

  for (let y = radius; y < height - radius; y++) {
    for (let x = radius; x < width - radius; x++) {
      const idx = (y * width + x) * 4;
      
      for (let c = 0; c < 3; c++) {
        // Compute local average
        let sum = 0;
        let count = 0;
        for (let dy = -radius; dy <= radius; dy++) {
          for (let dx = -radius; dx <= radius; dx++) {
            sum += src[((y + dy) * width + (x + dx)) * 4 + c];
            count++;
          }
        }
        const avg = sum / count;
        // Enhance difference from local average
        const diff = src[idx + c] - avg;
        result[idx + c] = clamp(src[idx + c] + diff * strength);
      }
    }
  }

  return new ImageData(result, width, height);
}

// ============================================================================
// Clone Stamp / Healing
// ============================================================================

export interface HealingStroke {
  sourceX: number;
  sourceY: number;
  targetX: number;
  targetY: number;
  radius: number;
  hardness: number; // 0-1
}

/**
 * Perform a single clone-stamp dab: copies pixels from source region to target
 * with feathered blending based on distance from center.
 */
export function cloneStampDab(
  ctx: CanvasRenderingContext2D,
  sourceCanvas: HTMLCanvasElement,
  sourceX: number,
  sourceY: number,
  targetX: number,
  targetY: number,
  radius: number,
  hardness: number = 0.7,
): void {
  const sourceCtx = sourceCanvas.getContext('2d');
  if (!sourceCtx) return;

  const r = Math.ceil(radius);
  const x0 = Math.max(0, targetX - r);
  const y0 = Math.max(0, targetY - r);
  const x1 = Math.min(ctx.canvas.width, targetX + r);
  const y1 = Math.min(ctx.canvas.height, targetY + r);

  // Get source pixels
  const srcOffX = sourceX - targetX;
  const srcOffY = sourceY - targetY;
  const srcX0 = Math.max(0, x0 + srcOffX);
  const srcY0 = Math.max(0, y0 + srcOffY);

  if (srcX0 >= sourceCanvas.width || srcY0 >= sourceCanvas.height) return;

  const w = Math.min(x1 - x0, sourceCanvas.width - srcX0);
  const h = Math.min(y1 - y0, sourceCanvas.height - srcY0);
  if (w <= 0 || h <= 0) return;

  const srcImageData = sourceCtx.getImageData(srcX0, srcY0, w, h);
  const dstImageData = ctx.getImageData(x0, y0, w, h);

  const srcData = srcImageData.data;
  const dstData = dstImageData.data;

  for (let py = 0; py < h; py++) {
    for (let px = 0; px < w; px++) {
      const dx = (x0 + px) - targetX;
      const dy = (y0 + py) - targetY;
      const dist = Math.sqrt(dx * dx + dy * dy);
      
      if (dist > radius) continue;

      // Feathered alpha based on distance and hardness
      const normalizedDist = dist / radius;
      const hardEdge = hardness;
      let alpha: number;
      if (normalizedDist <= hardEdge) {
        alpha = 1;
      } else {
        alpha = 1 - (normalizedDist - hardEdge) / (1 - hardEdge);
      }
      alpha = Math.max(0, Math.min(1, alpha));

      const idx = (py * w + px) * 4;
      dstData[idx]     = dstData[idx]     + (srcData[idx]     - dstData[idx])     * alpha;
      dstData[idx + 1] = dstData[idx + 1] + (srcData[idx + 1] - dstData[idx + 1]) * alpha;
      dstData[idx + 2] = dstData[idx + 2] + (srcData[idx + 2] - dstData[idx + 2]) * alpha;
    }
  }

  ctx.putImageData(dstImageData, x0, y0);
}

// ============================================================================
// Full Pipeline
// ============================================================================

/**
 * Run the complete adjustment pipeline on a source image.
 * Returns a fully processed ImageData ready for canvas rendering.
 */
export function processImage(
  sourceImageData: ImageData,
  adj: Adjustments,
): ImageData {
  const { width, height } = sourceImageData;

  // Step 1: Core per-pixel adjustments
  let result = applyAdjustments(sourceImageData, adj, width, height);

  // Step 2: Clarity (requires neighbor access)
  if (adj.clarity !== 0) {
    result = applyClarity(result, adj.clarity);
  }

  // Step 3: Sharpen (requires neighbor access)
  if (adj.sharpen > 0) {
    result = applySharpen(result, adj.sharpen);
  }

  return result;
}

/**
 * Render a source image onto a canvas with all adjustments applied.
 * Handles rotation and flip transforms.
 */
export function renderToCanvas(
  canvas: HTMLCanvasElement,
  sourceImage: HTMLImageElement | HTMLCanvasElement,
  adj: Adjustments,
): void {
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  if (!ctx) return;

  // Determine output dimensions after rotation
  const isRotated90 = adj.rotation === 90 || adj.rotation === 270;
  const srcW = sourceImage.width || (sourceImage as HTMLImageElement).naturalWidth;
  const srcH = sourceImage.height || (sourceImage as HTMLImageElement).naturalHeight;
  
  const outW = isRotated90 ? srcH : srcW;
  const outH = isRotated90 ? srcW : srcH;

  canvas.width = outW;
  canvas.height = outH;

  // Apply rotation and flip transforms
  ctx.save();
  ctx.translate(outW / 2, outH / 2);
  ctx.rotate((adj.rotation * Math.PI) / 180);
  if (adj.flipH) ctx.scale(-1, 1);
  if (adj.flipV) ctx.scale(1, -1);
  ctx.drawImage(sourceImage, -srcW / 2, -srcH / 2, srcW, srcH);
  ctx.restore();

  // Check if any adjustments are non-default
  const hasAdjustments =
    adj.exposure !== 0 || adj.contrast !== 0 || adj.highlights !== 0 ||
    adj.shadows !== 0 || adj.temperature !== 0 || adj.tint !== 0 ||
    adj.saturation !== 0 || adj.vibrance !== 0 || adj.sharpen > 0 ||
    adj.clarity !== 0 || adj.vignette > 0 || adj.grain > 0;

  if (hasAdjustments) {
    const imageData = ctx.getImageData(0, 0, outW, outH);
    const processed = processImage(imageData, adj);
    ctx.putImageData(processed, 0, 0);
  }
}

/**
 * Check if adjustments are at default (no edits applied).
 */
export function isDefault(adj: Adjustments): boolean {
  return (
    adj.exposure === 0 && adj.contrast === 0 && adj.highlights === 0 &&
    adj.shadows === 0 && adj.temperature === 0 && adj.tint === 0 &&
    adj.saturation === 0 && adj.vibrance === 0 && adj.sharpen === 0 &&
    adj.clarity === 0 && adj.vignette === 0 && adj.grain === 0 &&
    adj.rotation === 0 && !adj.flipH && !adj.flipV
  );
}

/**
 * Merge a partial preset into existing adjustments.
 */
export function mergePreset(base: Adjustments, preset: Partial<Adjustments>): Adjustments {
  return { ...base, ...preset };
}
