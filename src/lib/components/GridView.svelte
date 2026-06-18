<script lang="ts">
  import type { MediaItem, BurstGroup } from '$lib/types';
  import { toAssetUrl } from '$lib/services/tauri-bridge';
  import { getCategoryColor, getCategoryName } from '$lib/types';
  import CategoryBadge from './CategoryBadge.svelte';

  let {
    items,
    originalItems,
    selectedIndex,
    thumbnailSize = 200,
    onSelect,
    onDoubleClick,
  }: {
    items: (MediaItem | BurstGroup)[];
    originalItems: MediaItem[];
    selectedIndex: number;
    thumbnailSize?: number;
    onSelect: (index: number) => void;
    onDoubleClick: (index: number, burstItems?: MediaItem[]) => void;
  } = $props();

  let gridContainer: HTMLDivElement;

  // Helper to check if a grid item is selected based on original selected index
  function isItemSelected(item: MediaItem | BurstGroup, selIndex: number): boolean {
    const selectedItem = originalItems[selIndex];
    if (!selectedItem) return false;

    if ('leadItem' in item) {
      return item.items.some(x => x.id === selectedItem.id);
    } else {
      return item.id === selectedItem.id;
    }
  }

  // Find index in original list
  function findOriginalIndex(item: MediaItem | BurstGroup): number {
    const target = 'leadItem' in item ? item.leadItem : item;
    return originalItems.findIndex(x => x.id === target.id);
  }

  // Scroll selected item into view
  $effect(() => {
    if (gridContainer && selectedIndex >= 0) {
      // Find element that has data-selected="true"
      const el = gridContainer.querySelector('[data-selected="true"]');
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
  {#each items as item (item.id)}
    {@const isBurst = 'leadItem' in item}
    {@const displayItem = isBurst ? (item as BurstGroup).leadItem : (item as MediaItem)}
    {@const origIdx = findOriginalIndex(item)}
    {@const selected = isItemSelected(item, selectedIndex)}

    <button
      class="grid-item"
      class:selected={selected}
      class:burst-stack={isBurst}
      data-selected={selected ? "true" : "false"}
      onclick={() => onSelect(origIdx)}
      ondblclick={() => onDoubleClick(origIdx, isBurst ? (item as BurstGroup).items : undefined)}
    >
      <div class="thumb-container">
        {#if displayItem.thumbnail_path}
          <img
            src={toAssetUrl(displayItem.thumbnail_path)}
            alt={displayItem.file_name}
            class="thumb-img"
            loading="lazy"
            draggable="false"
          />
        {:else}
          <div class="thumb-placeholder">
            {#if displayItem.file_type === 'video'}
              <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <polygon points="5,3 19,12 5,21"/>
              </svg>
            {:else}
              <div class="skeleton" style="width: 100%; height: 100%;"></div>
            {/if}
          </div>
        {/if}

        <!-- Category badge overlay -->
        {#if displayItem.category_id}
          <div class="grid-badge">
            <CategoryBadge categoryId={displayItem.category_id} compact />
          </div>
        {/if}

        <!-- Star rating overlay -->
        {#if displayItem.star_rating > 0}
          <div class="grid-stars">
            {#each Array(displayItem.star_rating) as _}
              <span class="star">★</span>
            {/each}
          </div>
        {/if}

        <!-- Burst group count badge -->
        {#if isBurst}
          <div class="burst-badge">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
            </svg>
            <span>{(item as BurstGroup).items.length} Burst</span>
          </div>
        {/if}

        <!-- Video indicator -->
        {#if displayItem.file_type === 'video'}
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
    gap: var(--space-3);
    padding: var(--space-4);
    overflow-y: auto;
    height: 100%;
    align-content: start;
  }

  .grid-item {
    aspect-ratio: 1;
    border-radius: var(--radius-md);
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
    z-index: 5;
    box-shadow: var(--shadow-lg);
  }

  .grid-item.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent), var(--shadow-glow);
    transform: scale(1.02);
    z-index: 4;
  }

  /* Smart Burst Stack Effect */
  .grid-item.burst-stack::before,
  .grid-item.burst-stack::after {
    content: '';
    position: absolute;
    inset: 0;
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: var(--shadow-sm);
    z-index: -1;
    transition: transform 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
    pointer-events: none;
  }

  .grid-item.burst-stack::before {
    transform: rotate(-3deg) scale(0.97);
  }

  .grid-item.burst-stack::after {
    transform: rotate(3deg) scale(0.97);
    z-index: -2;
  }

  .grid-item.burst-stack:hover::before {
    transform: rotate(-6deg) scale(0.99);
  }

  .grid-item.burst-stack:hover::after {
    transform: rotate(6deg) scale(0.99);
  }

  .thumb-container {
    width: 100%;
    height: 100%;
    position: relative;
    overflow: hidden;
    border-radius: var(--radius-md);
  }

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
    box-shadow: inset 0 0 10px rgba(0,0,0,0.15);
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
    top: var(--space-2);
    right: var(--space-2);
    z-index: 2;
  }

  .grid-stars {
    position: absolute;
    bottom: var(--space-2);
    left: var(--space-2);
    display: flex;
    gap: 1px;
    z-index: 2;
  }

  .star {
    color: var(--color-star);
    font-size: 10px;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
  }

  .burst-badge {
    position: absolute;
    top: var(--space-2);
    left: var(--space-2);
    background: rgba(168, 85, 247, 0.85); /* Purple accent */
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: white;
    font-size: 9px;
    font-weight: 700;
    padding: 2px var(--space-2);
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
    gap: 4px;
    z-index: 2;
    box-shadow: var(--shadow-sm);
  }

  .video-indicator {
    position: absolute;
    bottom: var(--space-2);
    right: var(--space-2);
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
