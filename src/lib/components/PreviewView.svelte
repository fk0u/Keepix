<script lang="ts">
  import type { MediaItem } from '$lib/types';
  import { toAssetUrl } from '$lib/services/tauri-bridge';
  import { getCategoryColor, getCategoryName } from '$lib/types';
  import CategoryBadge from './CategoryBadge.svelte';
  import VideoPlayer from './VideoPlayer.svelte';
  import { compareMode, syncZoom, diagnosticsMode, showHistogram } from '$lib/stores/media';
  import DiagnosticsCanvas from './DiagnosticsCanvas.svelte';
  import Histogram from './Histogram.svelte';

  let {
    item,
    items,
    selectedIndex,
    onNavigate,
  }: {
    item: MediaItem | null;
    items: MediaItem[];
    selectedIndex: number;
    onNavigate: (index: number) => void;
  } = $props();

  let sharedZoomed = $state(false);
  let sharedZoomPos = $state({ x: 50, y: 50 });
  let zoomedPanelId = $state<string | null>(null);

  // Reset zoom when items or selectedIndex changes
  $effect(() => {
    const _mode = $compareMode;
    const _idx = selectedIndex;
    sharedZoomed = false;
    zoomedPanelId = null;
  });

  // Calculate items being compared dynamically based on compareMode
  let activeItems = $derived.by(() => {
    if (!item) return [];
    if ($compareMode === '2-up') {
      return [
        items[selectedIndex] ?? null,
        items[selectedIndex + 1] ?? null,
      ].filter(Boolean) as MediaItem[];
    }
    if ($compareMode === '4-up') {
      return [
        items[selectedIndex] ?? null,
        items[selectedIndex + 1] ?? null,
        items[selectedIndex + 2] ?? null,
        items[selectedIndex + 3] ?? null,
      ].filter(Boolean) as MediaItem[];
    }
    return [item];
  });

  function handleMouseMove(e: MouseEvent) {
    if (!sharedZoomed) return;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    sharedZoomPos = {
      x: ((e.clientX - rect.left) / rect.width) * 100,
      y: ((e.clientY - rect.top) / rect.height) * 100,
    };
  }

  function handlePanelClick(e: MouseEvent, itemId: string) {
    e.stopPropagation();
    if (sharedZoomed) {
      sharedZoomed = false;
      zoomedPanelId = null;
    } else {
      sharedZoomed = true;
      zoomedPanelId = itemId;
    }
  }
</script>

<div class="preview-view">
  <!-- Panels Grid -->
  <div class="panels-grid compare-{$compareMode}">
    {#each activeItems as activeItem, idx (activeItem.id)}
      <div
        class="preview-panel"
        class:active-panel={idx === 0}
        class:zoomed={sharedZoomed && ($syncZoom || zoomedPanelId === activeItem.id)}
        onclick={(e) => activeItem.file_type === 'photo' && handlePanelClick(e, activeItem.id)}
        onmousemove={handleMouseMove}
        role="presentation"
      >
        {#if activeItem.file_type === 'photo'}
          {#if $diagnosticsMode !== 'none'}
            <DiagnosticsCanvas
              src={toAssetUrl(activeItem.preview_path || activeItem.file_path)}
              mode={$diagnosticsMode}
              style={sharedZoomed && ($syncZoom || zoomedPanelId === activeItem.id)
                ? `transform-origin: ${sharedZoomPos.x}% ${sharedZoomPos.y}%; transform: scale(2.5);`
                : ''}
            />
          {:else}
            <img
              src={toAssetUrl(activeItem.preview_path || activeItem.file_path)}
              alt={activeItem.file_name}
              class="preview-img"
              style={sharedZoomed && ($syncZoom || zoomedPanelId === activeItem.id)
                ? `transform-origin: ${sharedZoomPos.x}% ${sharedZoomPos.y}%; transform: scale(2.5);`
                : ''}
              draggable="false"
            />
          {/if}
        {:else}
          <VideoPlayer item={activeItem} />
        {/if}

        <!-- Panel details overlay -->
        <div class="panel-overlay">
          <span class="panel-filename">{activeItem.file_name}</span>
          {#if activeItem.category_id}
            <CategoryBadge categoryId={activeItem.category_id} compact />
          {/if}
        </div>
      </div>
    {/each}
  </div>

  <!-- Navigation arrows -->
  <button
    class="nav-arrow nav-prev"
    onclick={(e) => { e.stopPropagation(); onNavigate(selectedIndex - 1); }}
    disabled={selectedIndex <= 0}
  >
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polyline points="15 18 9 12 15 6"/>
    </svg>
  </button>
  <button
    class="nav-arrow nav-next"
    onclick={(e) => { e.stopPropagation(); onNavigate(selectedIndex + 1); }}
    disabled={selectedIndex >= items.length - 1}
  >
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <polyline points="9 18 15 12 9 6"/>
    </svg>
  </button>

  <!-- Filmstrip -->
  <div class="filmstrip">
    {#each items as filmItem, idx (filmItem.id)}
      <button
        class="filmstrip-item"
        class:active={idx === selectedIndex}
        class:categorized={!!filmItem.category_id}
        onclick={() => onNavigate(idx)}
        style={filmItem.category_id ? `border-color: ${getCategoryColor(filmItem.category_id)}` : ''}
      >
        {#if filmItem.thumbnail_path}
          <img
            src={toAssetUrl(filmItem.thumbnail_path)}
            alt=""
            loading="lazy"
            draggable="false"
          />
        {:else}
          <div class="filmstrip-placeholder"></div>
        {/if}
      </button>
    {/each}
  </div>

  <!-- Histogram Overlay -->
  {#if $showHistogram && item && item.file_type === 'photo'}
    <div class="histogram-overlay-container">
      <Histogram src={toAssetUrl(item.preview_path || item.file_path)} />
    </div>
  {/if}
</div>


<style>
  .preview-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    position: relative;
    background: var(--bg-primary);
  }

  .panels-grid {
    flex: 1;
    display: grid;
    gap: var(--space-2);
    padding: var(--space-2);
    background: var(--bg-primary);
    overflow: hidden;
  }

  .panels-grid.compare-single {
    grid-template-columns: 1fr;
    grid-template-rows: 1fr;
  }

  .panels-grid.compare-2-up {
    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr;
  }

  .panels-grid.compare-4-up {
    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr 1fr;
  }

  .preview-panel {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    background: #000;
    border-radius: var(--radius-lg);
    border: 2px solid transparent;
    transition: border-color var(--transition-base);
    cursor: zoom-in;
  }

  .preview-panel.zoomed {
    cursor: zoom-out;
  }

  .preview-panel.active-panel {
    border-color: var(--border-strong);
  }

  .preview-img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    transition: transform 0.1s ease;
    user-select: none;
  }

  .panel-overlay {
    position: absolute;
    bottom: var(--space-3);
    left: var(--space-3);
    right: var(--space-3);
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(10px);
    padding: 6px var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
    pointer-events: none;
    z-index: 5;
  }

  .panel-filename {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 80%;
  }

  /* Navigation arrows */
  .nav-arrow {
    position: absolute;
    top: calc(50% - 40px);
    transform: translateY(-50%);
    z-index: 5;
    background: rgba(0, 0, 0, 0.4);
    border: none;
    border-radius: var(--radius-full);
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  .preview-view:hover .nav-arrow {
    opacity: 0.7;
  }

  .nav-arrow:hover {
    opacity: 1 !important;
    background: rgba(0, 0, 0, 0.6);
  }

  .nav-arrow:disabled {
    opacity: 0 !important;
    cursor: default;
  }

  .nav-prev {
    left: var(--space-4);
  }

  .nav-next {
    right: var(--space-4);
  }

  /* Filmstrip */
  .filmstrip {
    display: flex;
    gap: 2px;
    padding: var(--space-1);
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-subtle);
    overflow-x: auto;
    overflow-y: hidden;
    height: var(--filmstrip-height);
    flex-shrink: 0;
    align-items: center;
  }

  .filmstrip-item {
    flex-shrink: 0;
    width: 60px;
    height: 60px;
    border-radius: var(--radius-sm);
    overflow: hidden;
    cursor: pointer;
    border: 2px solid transparent;
    transition: all var(--transition-fast);
    background: var(--bg-tertiary);
    padding: 0;
  }

  .filmstrip-item:hover {
    border-color: var(--border-strong);
    transform: scale(1.05);
  }

  .filmstrip-item.active {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }

  .filmstrip-item.categorized {
    border-width: 2px;
  }

  .filmstrip-item img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .filmstrip-placeholder {
    width: 100%;
    height: 100%;
    background: var(--bg-tertiary);
  }

  .histogram-overlay-container {
    position: absolute;
    top: var(--space-4);
    right: var(--space-4);
    z-index: 100;
    pointer-events: none;
  }
</style>
