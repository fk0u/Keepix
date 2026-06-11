<script lang="ts">
  import type { MediaItem } from '$lib/types';
  import { toAssetUrl } from '$lib/services/tauri-bridge';
  import { getCategoryColor, getCategoryName } from '$lib/types';
  import CategoryBadge from './CategoryBadge.svelte';

  let {
    items,
    selectedIndex,
    thumbnailSize = 200,
    onSelect,
    onDoubleClick,
  }: {
    items: MediaItem[];
    selectedIndex: number;
    thumbnailSize?: number;
    onSelect: (index: number) => void;
    onDoubleClick: (index: number) => void;
  } = $props();

  let gridContainer: HTMLDivElement;

  // Scroll selected item into view
  $effect(() => {
    if (gridContainer && selectedIndex >= 0) {
      const el = gridContainer.querySelector(`[data-index="${selectedIndex}"]`);
      if (el) {
        el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
      }
    }
  });
</script>

<div
  class="grid-view"
  bind:this={gridContainer}
  style="--thumb-size: {thumbnailSize}px"
>
  {#each items as item, idx (item.id)}
    <button
      class="grid-item"
      class:selected={idx === selectedIndex}
      data-index={idx}
      onclick={() => onSelect(idx)}
      ondblclick={() => onDoubleClick(idx)}
    >
      <div class="thumb-container">
        {#if item.thumbnail_path}
          <img
            src={toAssetUrl(item.thumbnail_path)}
            alt={item.file_name}
            class="thumb-img"
            loading="lazy"
            draggable="false"
          />
        {:else}
          <div class="thumb-placeholder">
            {#if item.file_type === 'video'}
              <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <polygon points="5,3 19,12 5,21"/>
              </svg>
            {:else}
              <div class="skeleton" style="width: 100%; height: 100%;"></div>
            {/if}
          </div>
        {/if}

        <!-- Category badge overlay -->
        {#if item.category_id}
          <div class="grid-badge">
            <CategoryBadge categoryId={item.category_id} compact />
          </div>
        {/if}

        <!-- Star rating overlay -->
        {#if item.star_rating > 0}
          <div class="grid-stars">
            {#each Array(item.star_rating) as _}
              <span class="star">★</span>
            {/each}
          </div>
        {/if}

        <!-- Video indicator -->
        {#if item.file_type === 'video'}
          <div class="video-indicator">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="white">
              <polygon points="5,3 19,12 5,21"/>
            </svg>
          </div>
        {/if}
      </div>
    </button>
  {/each}
</div>

<style>
  .grid-view {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(var(--thumb-size), 1fr));
    gap: var(--space-1);
    padding: var(--space-2);
    overflow-y: auto;
    height: 100%;
    align-content: start;
  }

  .grid-item {
    aspect-ratio: 1;
    border-radius: var(--radius-md);
    overflow: hidden;
    cursor: pointer;
    border: 2px solid transparent;
    transition: all 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
    background: var(--bg-tertiary);
    position: relative;
    padding: 0;
    font-family: var(--font-sans);
    box-shadow: var(--shadow-sm);
  }

  .grid-item:hover {
    border-color: rgba(255, 255, 255, 0.3);
    transform: scale(1.03) translateY(-2px);
    z-index: 1;
    box-shadow: var(--shadow-md);
  }

  .grid-item.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent), var(--shadow-glow);
    transform: scale(1.02);
  }

  .thumb-container {
    width: 100%;
    height: 100%;
    position: relative;
    overflow: hidden;
  }

  /* Add an inner shadow/vignette to thumbnails for depth */
  .thumb-container::after {
    content: '';
    position: absolute;
    inset: 0;
    box-shadow: inset 0 0 20px rgba(0,0,0,0.4);
    pointer-events: none;
    z-index: 1;
    transition: box-shadow var(--transition-base);
  }

  .grid-item:hover .thumb-container::after {
    box-shadow: inset 0 0 10px rgba(0,0,0,0.1);
  }

  .thumb-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.4s cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  .grid-item:hover .thumb-img {
    transform: scale(1.05);
  }

  .thumb-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-tertiary);
    color: var(--text-tertiary);
  }

  .grid-badge {
    position: absolute;
    top: var(--space-1);
    right: var(--space-1);
    z-index: 2;
  }

  .grid-stars {
    position: absolute;
    bottom: var(--space-1);
    left: var(--space-1);
    display: flex;
    gap: 1px;
    z-index: 2;
  }

  .star {
    color: var(--color-star);
    font-size: 10px;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
  }

  .video-indicator {
    position: absolute;
    bottom: var(--space-1);
    right: var(--space-1);
    background: rgba(0, 0, 0, 0.6);
    border-radius: var(--radius-full);
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2;
  }
</style>
