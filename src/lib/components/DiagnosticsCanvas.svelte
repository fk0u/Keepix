<script lang="ts">
  import type { DiagnosticsMode } from '$lib/types';
  import { onMount } from 'svelte';

  let {
    src,
    mode = 'none',
    style = '',
  }: {
    src: string;
    mode: DiagnosticsMode;
    style?: string;
  } = $props();

  let canvasEl = $state<HTMLCanvasElement | null>(null);
  let loading = $state(false);
  let imageObj: HTMLImageElement | null = null;

  $effect(() => {
    if (src) {
      loadImage(src);
    }
  });

  $effect(() => {
    // Re-apply filter if mode changes
    const _mode = mode;
    if (imageObj && canvasEl) {
      applyDiagnostics();
    }
  });

  function loadImage(imageUrl: string) {
    loading = true;
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.src = imageUrl;
    img.onload = () => {
      imageObj = img;
      loading = false;
      applyDiagnostics();
    };
    img.onerror = () => {
      loading = false;
      imageObj = null;
    };
  }

  function applyDiagnostics() {
    if (!canvasEl || !imageObj) return;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;

    // Limit maximum size of canvas for focus peaking / clipping to ensure high performance
    const maxDim = 960;
    let w = imageObj.width;
    let h = imageObj.height;

    if (w > maxDim || h > maxDim) {
      if (w > h) {
        h = Math.round((h * maxDim) / w);
        w = maxDim;
      } else {
        w = Math.round((w * maxDim) / h);
        h = maxDim;
      }
    }

    canvasEl.width = w;
    canvasEl.height = h;

    // Draw original image
    ctx.drawImage(imageObj, 0, 0, w, h);

    if (mode === 'none') {
      return; // Normal drawing is sufficient
    }

    // Run custom pixel shaders
    try {
      const imgData = ctx.getImageData(0, 0, w, h);
      const input = imgData.data;
      
      // Create output buffer
      const outputImgData = ctx.createImageData(w, h);
      const output = outputImgData.data;

      if (mode === 'peaking') {
        runFocusPeaking(w, h, input, output);
      } else if (mode === 'zebra') {
        runExposureZebra(w, h, input, output);
      }

      ctx.putImageData(outputImgData, 0, 0);
    } catch (e) {
      console.error('Failed to apply diagnostics filter:', e);
    }
  }

  function runFocusPeaking(width: number, height: number, input: Uint8ClampedArray, output: Uint8ClampedArray) {
    // Sobel Kernels
    const gx = [
      [-1, 0, 1],
      [-2, 0, 2],
      [-1, 0, 1]
    ];
    const gy = [
      [-1, -2, -1],
      [0, 0, 0],
      [1, 2, 1]
    ];

    const threshold = 38; // Contrast edge sensitivity

    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const idx = (y * width + x) * 4;

        // Border handling
        if (y === 0 || y === height - 1 || x === 0 || x === width - 1) {
          output[idx] = input[idx] * 0.5;
          output[idx + 1] = input[idx + 1] * 0.5;
          output[idx + 2] = input[idx + 2] * 0.5;
          output[idx + 3] = 255;
          continue;
        }

        let valX = 0;
        let valY = 0;

        for (let ky = -1; ky <= 1; ky++) {
          for (let kx = -1; kx <= 1; kx++) {
            const pixelIdx = ((y + ky) * width + (x + kx)) * 4;
            const r = input[pixelIdx];
            const g = input[pixelIdx + 1];
            const b = input[pixelIdx + 2];
            const luma = 0.299 * r + 0.587 * g + 0.114 * b;

            valX += luma * gx[ky + 1][kx + 1];
            valY += luma * gy[ky + 1][kx + 1];
          }
        }

        const mag = Math.sqrt(valX * valX + valY * valY);

        if (mag > threshold) {
          // Sharp edge: draw electric neon green peaking color
          output[idx] = 34;      // R
          output[idx + 1] = 197;  // G
          output[idx + 2] = 94;   // B
          output[idx + 3] = 255;
        } else {
          // Dim original background slightly so peaking stands out
          output[idx] = Math.round(input[idx] * 0.65);
          output[idx + 1] = Math.round(input[idx + 1] * 0.65);
          output[idx + 2] = Math.round(input[idx + 2] * 0.65);
          output[idx + 3] = 255;
        }
      }
    }
  }

  function runExposureZebra(width: number, height: number, input: Uint8ClampedArray, output: Uint8ClampedArray) {
    for (let i = 0; i < input.length; i += 4) {
      const r = input[i];
      const g = input[i + 1];
      const b = input[i + 2];

      // Highlight clipping (pure white > 250) -> Neon Red
      if (r >= 252 && g >= 252 && b >= 252) {
        output[i] = 239;
        output[i + 1] = 68;
        output[i + 2] = 68;
        output[i + 3] = 255;
      }
      // Shadow clipping (pure black < 4) -> Neon Blue
      else if (r <= 4 && g <= 4 && b <= 4) {
        output[i] = 59;
        output[i + 1] = 130;
        output[i + 2] = 246;
        output[i + 3] = 255;
      }
      // Normal pixel
      else {
        output[i] = r;
        output[i + 1] = g;
        output[i + 2] = b;
        output[i + 3] = 255;
      }
    }
  }
</script>

<div class="canvas-container">
  <canvas
    bind:this={canvasEl}
    class="preview-diagnostics-canvas"
    {style}
  ></canvas>
  {#if loading}
    <div class="canvas-loader">
      <div class="loader-spinner"></div>
    </div>
  {/if}
</div>

<style>
  .canvas-container {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }

  .preview-diagnostics-canvas {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    transition: transform 0.1s ease;
    user-select: none;
  }

  .canvas-loader {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.4);
    z-index: 10;
  }

  .loader-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-radius: 50%;
    border-top-color: var(--accent);
    animation: spin 1s ease-in-out infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
