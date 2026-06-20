<script lang="ts">
  import { t } from '$lib/i18n';

  let {
    beforeSrc,
    afterCanvas,
    visible = false,
    onClose,
  }: {
    beforeSrc: string;
    afterCanvas: HTMLCanvasElement | null;
    visible: boolean;
    onClose: () => void;
  } = $props();

  let sliderPosition = $state(50);
  let isDragging = $state(false);
  let containerRef: HTMLDivElement | undefined = $state();
  let mode = $state<'slider' | 'side-by-side'>('slider');
  let afterDataUrl = $state('');

  // Convert canvas to data URL when canvas changes
  $effect(() => {
    if (afterCanvas) {
      try {
        afterDataUrl = afterCanvas.toDataURL('image/png');
      } catch {
        afterDataUrl = '';
      }
    }
  });

  function handleMouseDown(e: MouseEvent) {
    e.preventDefault();
    isDragging = true;
    updatePosition(e);
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging) return;
    updatePosition(e);
  }

  function handleMouseUp() {
    isDragging = false;
  }

  function handleTouchStart(e: TouchEvent) {
    e.preventDefault();
    isDragging = true;
    updatePositionTouch(e);
  }

  function handleTouchMove(e: TouchEvent) {
    if (!isDragging) return;
    updatePositionTouch(e);
  }

  function updatePosition(e: MouseEvent) {
    if (!containerRef) return;
    const rect = containerRef.getBoundingClientRect();
    const x = e.clientX - rect.left;
    sliderPosition = Math.max(2, Math.min(98, (x / rect.width) * 100));
  }

  function updatePositionTouch(e: TouchEvent) {
    if (!containerRef || !e.touches[0]) return;
    const rect = containerRef.getBoundingClientRect();
    const x = e.touches[0].clientX - rect.left;
    sliderPosition = Math.max(2, Math.min(98, (x / rect.width) * 100));
  }

  function toggleMode() {
    mode = mode === 'slider' ? 'side-by-side' : 'slider';
  }
</script>

<svelte:window onmouseup={handleMouseUp} onmousemove={handleMouseMove} />

{#if visible}
  <div class="ba-overlay" role="dialog" aria-label="Before After Comparison">
    <!-- Controls bar -->
    <div class="ba-controls">
      <div class="ba-controls-left">
        <button class="ba-mode-btn" class:active={mode === 'slider'} onclick={() => mode = 'slider'}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="1" y="2" width="14" height="12" rx="1"/>
            <line x1="8" y1="2" x2="8" y2="14"/>
          </svg>
          {$t('edit.ba.slider')}
        </button>
        <button class="ba-mode-btn" class:active={mode === 'side-by-side'} onclick={() => mode = 'side-by-side'}>
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="1" y="2" width="6" height="12" rx="1"/>
            <rect x="9" y="2" width="6" height="12" rx="1"/>
          </svg>
          {$t('edit.ba.side_by_side')}
        </button>
      </div>
      <button class="ba-close-btn" onclick={onClose}>
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="4" y1="4" x2="14" y2="14"/>
          <line x1="14" y1="4" x2="4" y2="14"/>
        </svg>
      </button>
    </div>

    {#if mode === 'slider'}
      <!-- Slider comparison mode -->
      <div
        class="ba-container"
        bind:this={containerRef}
        onmousedown={handleMouseDown}
        ontouchstart={handleTouchStart}
        ontouchmove={handleTouchMove}
        ontouchend={handleMouseUp}
        role="slider"
        aria-valuenow={Math.round(sliderPosition)}
        aria-valuemin={0}
        aria-valuemax={100}
        tabindex={0}
      >
        <!-- After (edited) - full background -->
        <div class="ba-layer ba-after">
          {#if afterDataUrl}
            <img src={afterDataUrl} alt="After" draggable="false" />
          {/if}
        </div>

        <!-- Before (original) - clipped -->
        <div class="ba-layer ba-before" style="clip-path: inset(0 {100 - sliderPosition}% 0 0);">
          <img src={beforeSrc} alt="Before" draggable="false" />
        </div>

        <!-- Divider line -->
        <div class="ba-divider" style="left: {sliderPosition}%;">
          <div class="ba-divider-line"></div>
          <div class="ba-handle" class:dragging={isDragging}>
            <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="7 6 3 10 7 14"/>
              <polyline points="13 6 17 10 13 14"/>
            </svg>
          </div>
        </div>

        <!-- Labels -->
        <div class="ba-label ba-label-before" style="opacity: {sliderPosition > 15 ? 1 : 0}">
          {$t('edit.ba.before')}
        </div>
        <div class="ba-label ba-label-after" style="opacity: {sliderPosition < 85 ? 1 : 0}">
          {$t('edit.ba.after')}
        </div>
      </div>
    {:else}
      <!-- Side by side mode -->
      <div class="ba-side-by-side">
        <div class="ba-side">
          <div class="ba-side-label">{$t('edit.ba.before')}</div>
          <img src={beforeSrc} alt="Before" draggable="false" />
        </div>
        <div class="ba-side">
          <div class="ba-side-label">{$t('edit.ba.after')}</div>
          {#if afterDataUrl}
            <img src={afterDataUrl} alt="After" draggable="false" />
          {/if}
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .ba-overlay {
    position: absolute;
    inset: 0;
    z-index: 50;
    background: rgba(0, 0, 0, 0.95);
    display: flex;
    flex-direction: column;
    animation: baFadeIn 0.2s ease-out;
  }

  @keyframes baFadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .ba-controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    background: rgba(30, 30, 30, 0.9);
    backdrop-filter: blur(12px);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    flex-shrink: 0;
    z-index: 2;
  }

  .ba-controls-left {
    display: flex;
    gap: 4px;
  }

  .ba-mode-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: var(--text-secondary);
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .ba-mode-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }

  .ba-mode-btn.active {
    background: var(--accent-soft);
    border-color: var(--accent);
    color: white;
  }

  .ba-close-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.15s;
  }

  .ba-close-btn:hover {
    background: rgba(239, 68, 68, 0.3);
    border-color: rgba(239, 68, 68, 0.5);
    color: white;
  }

  /* Slider Mode */
  .ba-container {
    flex: 1;
    position: relative;
    overflow: hidden;
    cursor: ew-resize;
    user-select: none;
  }

  .ba-layer {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .ba-layer img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    user-select: none;
    pointer-events: none;
  }

  .ba-divider {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 0;
    transform: translateX(-50%);
    z-index: 5;
  }

  .ba-divider-line {
    position: absolute;
    top: 0;
    bottom: 0;
    left: 50%;
    width: 2px;
    transform: translateX(-50%);
    background: white;
    box-shadow: 0 0 8px rgba(0, 0, 0, 0.5);
  }

  .ba-handle {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.15);
    backdrop-filter: blur(12px);
    border: 2px solid white;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.4);
    transition: transform 0.15s, background 0.15s;
  }

  .ba-handle:hover,
  .ba-handle.dragging {
    transform: translate(-50%, -50%) scale(1.15);
    background: rgba(255, 255, 255, 0.25);
  }

  .ba-label {
    position: absolute;
    bottom: 20px;
    padding: 5px 14px;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 20px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    color: rgba(255, 255, 255, 0.8);
    transition: opacity 0.2s;
    z-index: 3;
    pointer-events: none;
  }

  .ba-label-before {
    left: 20px;
  }

  .ba-label-after {
    right: 20px;
  }

  /* Side-by-side Mode */
  .ba-side-by-side {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2px;
    padding: 8px;
    overflow: hidden;
  }

  .ba-side {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #000;
    border-radius: 8px;
    overflow: hidden;
  }

  .ba-side img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }

  .ba-side-label {
    position: absolute;
    top: 12px;
    left: 12px;
    padding: 4px 12px;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 16px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    color: rgba(255, 255, 255, 0.7);
    z-index: 2;
  }
</style>
