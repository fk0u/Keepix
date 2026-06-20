<script lang="ts">
  import type { MediaItem, BurstGroup } from '$lib/types';
  import { getCategoryColor, getCategoryName } from '$lib/types';
  import CategoryBadge from './CategoryBadge.svelte';
  import { getImageDataUri, prefetchImages } from '$lib/services/image-cache';
  import { onMount } from 'svelte';

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

  // Track loaded image data URIs reactively
  let loadedImages = $state<Record<string, string>>({});

  function formatDuration(seconds: number): string {
    if (isNaN(seconds) || seconds === Infinity) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
  }

  function getVideoDurationText(item: MediaItem): string | null {
    if (item.file_type !== 'video' || !item.exif_json) return null;
    try {
      const meta = JSON.parse(item.exif_json);
      if (meta && typeof meta.duration === 'number') {
        return formatDuration(meta.duration);
      }
    } catch {
      // ignore
    }
    return null;
  }

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
      const el = gridContainer.querySelector('[data-selected="true"]');
      if (el) {
        el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
      }
    }
  });

  // Load image via cache when element enters viewport
  function lazyLoadAction(node: HTMLElement, path: string | null) {
    if (!path) return;

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            loadImage(path);
            observer.unobserve(node);
          }
        }
      },
      { root: gridContainer, rootMargin: '200px' }
    );

    observer.observe(node);

    return {
      destroy() {
        observer.unobserve(node);
        observer.disconnect();
      },
    };
  }

  async function loadImage(path: string) {
    if (loadedImages[path]) return;
    const dataUri = await getImageDataUri(path);
    if (dataUri) {
      loadedImages = { ...loadedImages, [path]: dataUri };
    }
  }

  // Prefetch visible items when items array changes
  $effect(() => {
    if (items.length > 0) {
      const paths = items.slice(0, 50).map(item => {
        const displayItem = 'leadItem' in item ? item.leadItem : item;
        return displayItem.thumbnail_path;
      }).filter((p): p is string => !!p);
      
      prefetchImages(paths);
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
    {@const thumbPath = displayItem.thumbnail_path}

    <button
      class="grid-item"
      class:selected={selected}
      class:burst-stack={isBurst}
      data-selected={selected ? "true" : "false"}
      onclick={() => onSelect(origIdx)}
      ondblclick={() => onDoubleClick(origIdx, isBurst ? (item as BurstGroup).items : undefined)}
    >
      <div class="thumb-container" use:lazyLoadAction={thumbPath}>
        {#if thumbPath && loadedImages[thumbPath]}
          <img
            src={loadedImages[thumbPath]}
            alt={displayItem.file_name}
            class="thumb-img"
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

        <!-- Color label indicator -->
        {#if displayItem.color_label}
          <div class="color-label-dot" style="--label-color: {
            displayItem.color_label === 'red' ? '#ef4444' :
            displayItem.color_label === 'orange' ? '#f97316' :
            displayItem.color_label === 'yellow' ? '#eab308' :
            displayItem.color_label === 'green' ? '#22c55e' :
            displayItem.color_label === 'blue' ? '#3b82f6' :
            displayItem.color_label === 'purple' ? '#a855f7' : 'transparent'
          }"></div>
        {/if}

        <!-- Video indicator -->
        {#if displayItem.file_type === 'video'}
          {@const durationText = getVideoDurationText(displayItem)}
          <div class="video-indicator" class:has-duration={!!durationText} class:unsupported-video={displayItem.width === -1}>
            <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor">
              <polygon points="5,3 19,12 5,21"/>
            </svg>
            {#if durationText}
              <span class="video-duration-text">{durationText}</span>
            {/if}
            {#if displayItem.width === -1}
              <span class="video-unsupported-tag" title="Format tidak didukung di pratinjau langsung">⚠️ UNSUPPORTED</span>
            {/if}
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

  .color-label-dot {
    position: absolute;
    bottom: var(--space-2);
    right: var(--space-2);
    width: 10px;
    height: 10px;
    border-radius: var(--radius-full);
    background-color: var(--label-color);
    border: 1.5px solid rgba(255, 255, 255, 0.4);
    z-index: 2;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
  }

  .video-indicator {
    position: absolute;
    bottom: var(--space-2);
    right: var(--space-2);
    background: rgba(0, 0, 0, 0.75);
    border-radius: var(--radius-full);
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.1);
    transition: all var(--transition-fast);
    color: white;
  }

  .video-indicator.has-duration {
    width: auto;
    height: 18px;
    border-radius: var(--radius-sm);
    padding: 0 var(--space-2);
    gap: var(--space-1);
  }

  .video-indicator.unsupported-video {
    border-color: #ef4444;
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
  }

  .video-duration-text {
    font-size: 9px;
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--text-primary);
  }

  .video-unsupported-tag {
    font-size: 8px;
    font-weight: 700;
    letter-spacing: 0.3px;
    color: #f87171;
  }
</style>
