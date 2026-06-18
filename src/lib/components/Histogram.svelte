<script lang="ts">
  import { onMount } from 'svelte';

  let { src }: { src: string } = $props();

  let canvasEl = $state<HTMLCanvasElement | null>(null);
  let loading = $state(false);

  // Histograms
  let rData = new Uint32Array(256);
  let gData = new Uint32Array(256);
  let bData = new Uint32Array(256);
  let lData = new Uint32Array(256);

  let activeChannel = $state<'all' | 'luminance' | 'rgb'>('all');

  $effect(() => {
    if (src) {
      calculateHistogram(src);
    }
  });

  async function calculateHistogram(imageUrl: string) {
    if (!imageUrl) return;
    loading = true;

    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.src = imageUrl;

    img.onload = () => {
      // Create offscreen canvas for downsampling
      const offscreen = document.createElement('canvas');
      const sampleSize = 128; // Downsample for 60fps performance
      offscreen.width = sampleSize;
      offscreen.height = sampleSize;

      const ctx = offscreen.getContext('2d');
      if (!ctx) {
        loading = false;
        return;
      }

      ctx.drawImage(img, 0, 0, sampleSize, sampleSize);
      
      let imgData;
      try {
        imgData = ctx.getImageData(0, 0, sampleSize, sampleSize);
      } catch (e) {
        console.error('Failed to read pixel data for histogram:', e);
        loading = false;
        return;
      }

      const data = imgData.data;

      // Reset histogram counts
      rData.fill(0);
      gData.fill(0);
      bData.fill(0);
      lData.fill(0);

      // Extract channels
      for (let i = 0; i < data.length; i += 4) {
        const r = data[i];
        const g = data[i + 1];
        const b = data[i + 2];
        // Standard Rec. 601 luma formula
        const l = Math.round(0.299 * r + 0.587 * g + 0.114 * b);

        rData[r]++;
        gData[g]++;
        bData[b]++;
        lData[l]++;
      }

      loading = false;
      drawHistogram();
    };

    img.onerror = () => {
      loading = false;
    };
  }

  function drawHistogram() {
    if (!canvasEl) return;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;

    const width = canvasEl.width;
    const height = canvasEl.height;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Find peak value for scaling
    let peak = 0;
    if (activeChannel === 'luminance') {
      for (let i = 0; i < 256; i++) {
        if (lData[i] > peak) peak = lData[i];
      }
    } else if (activeChannel === 'rgb') {
      for (let i = 0; i < 256; i++) {
        if (rData[i] > peak) peak = rData[i];
        if (gData[i] > peak) peak = gData[i];
        if (bData[i] > peak) peak = bData[i];
      }
    } else {
      // combined/all
      for (let i = 0; i < 256; i++) {
        if (rData[i] > peak) peak = rData[i];
        if (gData[i] > peak) peak = gData[i];
        if (bData[i] > peak) peak = bData[i];
        if (lData[i] > peak) peak = lData[i];
      }
    }

    if (peak === 0) return;

    // Helper to draw a single channel curve
    const drawChannel = (data: Uint32Array, color: string, fillGradient: string) => {
      ctx.beginPath();
      ctx.moveTo(0, height);

      for (let i = 0; i < 256; i++) {
        const x = (i / 255) * width;
        // Apply dynamic scale (using log or square root can compress spikes, but linear is standard)
        // Let's use sqrt scaling for better visualization of small details
        const ratio = Math.sqrt(data[i]) / Math.sqrt(peak);
        const y = height - ratio * height * 0.95; // Leave 5% padding at top
        ctx.lineTo(x, y);
      }

      ctx.lineTo(width, height);
      ctx.closePath();

      // Fill
      const grad = ctx.createLinearGradient(0, 0, 0, height);
      grad.addColorStop(0, fillGradient);
      grad.addColorStop(1, 'rgba(0, 0, 0, 0)');
      ctx.fillStyle = grad;
      ctx.fill();

      // Stroke line
      ctx.strokeStyle = color;
      ctx.lineWidth = 1.5;
      ctx.stroke();
    };

    ctx.globalCompositeOperation = 'screen';

    // Draw channels based on selection
    if (activeChannel === 'all' || activeChannel === 'rgb') {
      drawChannel(rData, 'rgba(239, 68, 68, 0.95)', 'rgba(239, 68, 68, 0.2)'); // Red
      drawChannel(gData, 'rgba(34, 197, 94, 0.95)', 'rgba(34, 197, 94, 0.2)'); // Green
      drawChannel(bData, 'rgba(59, 130, 246, 0.95)', 'rgba(59, 130, 246, 0.2)'); // Blue
    }

    if (activeChannel === 'all' || activeChannel === 'luminance') {
      // Draw Luminance (white/gray)
      drawChannel(lData, 'rgba(243, 244, 246, 0.95)', 'rgba(243, 244, 246, 0.15)');
    }
  }

  $effect(() => {
    // Redraw if channel changes
    const _chan = activeChannel;
    drawHistogram();
  });
</script>

<div class="histogram-panel">
  <div class="histogram-header">
    <span class="histogram-title">HISTOGRAM</span>
    <div class="channel-toggles">
      <button 
        class="toggle-btn" 
        class:active={activeChannel === 'all'} 
        onclick={() => activeChannel = 'all'}
      >
        All
      </button>
      <button 
        class="toggle-btn" 
        class:active={activeChannel === 'luminance'} 
        onclick={() => activeChannel = 'luminance'}
      >
        Luma
      </button>
      <button 
        class="toggle-btn" 
        class:active={activeChannel === 'rgb'} 
        onclick={() => activeChannel = 'rgb'}
      >
        RGB
      </button>
    </div>
  </div>

  <div class="canvas-wrapper">
    <canvas 
      bind:this={canvasEl} 
      width="256" 
      height="120"
      class="histogram-canvas"
    ></canvas>
    {#if loading}
      <div class="histogram-loader">Calculating...</div>
    {/if}
  </div>
</div>

<style>
  .histogram-panel {
    background: rgba(15, 15, 18, 0.7);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: var(--radius-lg);
    padding: var(--space-2);
    width: 272px;
    box-shadow: var(--shadow-lg), 0 0 20px rgba(0,0,0,0.5);
    user-select: none;
    pointer-events: auto;
  }

  .histogram-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-2);
  }

  .histogram-title {
    font-size: 9px;
    font-weight: 700;
    color: var(--text-tertiary);
    letter-spacing: 1.5px;
    font-family: var(--font-sans);
  }

  .channel-toggles {
    display: flex;
    gap: 2px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: var(--radius-sm);
    padding: 1px;
  }

  .toggle-btn {
    background: transparent;
    border: none;
    font-size: 9px;
    color: var(--text-tertiary);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-weight: 500;
    transition: all var(--transition-fast);
  }

  .toggle-btn:hover {
    color: var(--text-primary);
  }

  .toggle-btn.active {
    background: rgba(255, 255, 255, 0.1);
    color: white;
    font-weight: 600;
  }

  .canvas-wrapper {
    position: relative;
    width: 100%;
    height: 120px;
    background: rgba(0, 0, 0, 0.3);
    border-radius: var(--radius-md);
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.03);
  }

  .histogram-canvas {
    display: block;
    width: 100%;
    height: 100%;
  }

  .histogram-loader {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    font-family: var(--font-sans);
  }
</style>
