<script lang="ts">
  import type { MediaItem } from '$lib/types';
  import { toAssetUrl } from '$lib/services/tauri-bridge';
  import { getCategoryColor, getCategoryName } from '$lib/types';
  import CategoryBadge from './CategoryBadge.svelte';

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

  let isZoomed = $state(false);
  let zoomPos = $state({ x: 50, y: 50 });

  function handleMouseMove(e: MouseEvent) {
    if (!isZoomed) return;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    zoomPos = {
      x: ((e.clientX - rect.left) / rect.width) * 100,
      y: ((e.clientY - rect.top) / rect.height) * 100,
    };
  }

  function toggleZoom() {
    isZoomed = !isZoomed;
  }
</script>

<div class="preview-view">
  <!-- Main image -->
  <div
    class="preview-main"
    class:zoomed={isZoomed}
    onclick={toggleZoom}
    onmousemove={handleMouseMove}
    role="img"
    tabindex="-1"
  >
    {#if item}
      {#if item.file_type === 'photo'}
        <img
          src={toAssetUrl(item.preview_path || item.file_path)}
          alt={item.file_name}
          class="preview-img"
          style={isZoomed ? `transform-origin: ${zoomPos.x}% ${zoomPos.y}%; transform: scale(2.5);` : ''}
          draggable="false"
        />
      {:else}
        <div class="video-preview">
          <svg width="64" height="64" viewBox="0 0 24 24" fill="currentColor" opacity="0.3">
            <polygon points="5,3 19,12 5,21"/>
          </svg>
          <p>Video preview</p>
          <p class="text-sm">{item.file_name}</p>
        </div>
      {/if}

      <!-- Category indicator -->
      {#if item.category_id}
        <div class="preview-category">
          <CategoryBadge categoryId={item.category_id} />
        </div>
      {/if}

      <!-- Navigation arrows -->
      <button
        class="nav-arrow nav-prev"
        onclick|stopPropagation={() => onNavigate(selectedIndex - 1)}
        disabled={selectedIndex <= 0}
      >
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="15 18 9 12 15 6"/>
        </svg>
      </button>
      <button
        class="nav-arrow nav-next"
        onclick|stopPropagation={() => onNavigate(selectedIndex + 1)}
        disabled={selectedIndex >= items.length - 1}
      >
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="9 18 15 12 9 6"/>
        </svg>
      </button>
    {/if}
  </div>

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
</div>

<style>
  .preview-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
  }

  .preview-main {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    position: relative;
    background: var(--bg-primary);
    cursor: zoom-in;
  }

  .preview-main.zoomed {
    cursor: zoom-out;
  }

  .preview-img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    transition: transform 0.1s ease;
    user-select: none;
  }

  .video-preview {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-tertiary);
  }

  .text-sm {
    font-size: var(--text-sm);
    opacity: 0.5;
  }

  .preview-category {
    position: absolute;
    top: var(--space-4);
    right: var(--space-4);
    z-index: 5;
  }

  /* Navigation arrows */
  .nav-arrow {
    position: absolute;
    top: 50%;
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

  .preview-main:hover .nav-arrow {
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
</style>
