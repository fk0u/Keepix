<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { currentProject, scanProgress, isScanning } from '$lib/stores/project';
  import {
    mediaItems, totalItems, selectedIndex, viewMode, isLoading,
    currentItem, categoryFilter, uncategorizedOnly, categoryStats,
    loadMediaItems, loadCategoryStats, loadCategories,
    setCategoryForItem, setStarRatingForItem,
    navigateNext, navigatePrev, navigateTo, toggleViewMode,
    setFilterCategory, setFilterUncategorized, clearFilters,
    categories,
  } from '$lib/stores/media';
  import { toast } from '$lib/stores/toast';
  import { toAssetUrl } from '$lib/services/tauri-bridge';
  import * as bridge from '$lib/services/tauri-bridge';
  import { getCategoryName, getCategoryColor, formatFileSize, type ExifData } from '$lib/types';
  import GridView from '$lib/components/GridView.svelte';
  import PreviewView from '$lib/components/PreviewView.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import Toolbar from '$lib/components/Toolbar.svelte';
  import MetadataPanel from '$lib/components/MetadataPanel.svelte';
  import ProgressBar from '$lib/components/ProgressBar.svelte';
  import KeyboardShortcutsModal from '$lib/components/KeyboardShortcutsModal.svelte';

  let showMetadata = $state(false);
  let showShortcuts = $state(false);
  let exifData = $state<ExifData | null>(null);
  let thumbnailSize = $state(200);
  let categoryFlash = $state<number | null>(null);

  // Reactive: load EXIF when selected item changes
  let prevItemId = '';

  $effect(() => {
    const item = $currentItem;
    if (item && item.id !== prevItemId) {
      prevItemId = item.id;
      loadExif(item.id);
    }
  });

  // Reactive: reload items when filters change
  $effect(() => {
    // Subscribe to filter changes
    const _cat = $categoryFilter;
    const _uncat = $uncategorizedOnly;
    if ($currentProject) {
      loadMediaItems($currentProject.id);
    }
  });

  onMount(async () => {
    if (!$currentProject) {
      goto('/');
      return;
    }

    // If items aren't loaded yet, load them
    if ($mediaItems.length === 0) {
      await loadMediaItems($currentProject.id);
      await loadCategoryStats($currentProject.id);
    }
  });

  async function loadExif(mediaId: string) {
    try {
      exifData = await bridge.getMetadata(mediaId);
    } catch {
      exifData = null;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // Don't handle if typing in an input
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

    const item = $currentItem;
    const projectId = $currentProject?.id;

    switch (e.key) {
      // Category assignment
      case '1':
      case '2':
      case '3':
      case '4': {
        if (!item || !projectId) return;
        const catId = parseInt(e.key);
        setCategoryForItem(item.id, projectId, catId);
        flashCategory(catId);
        const name = getCategoryName(catId);
        toast.success(`${name}`, 1500);
        // Auto-advance to next image
        navigateNext();
        break;
      }

      // Navigation
      case 'ArrowRight':
      case 'ArrowDown':
        e.preventDefault();
        navigateNext();
        break;
      case 'ArrowLeft':
      case 'ArrowUp':
        e.preventDefault();
        navigatePrev();
        break;

      // View mode
      case ' ':
        e.preventDefault();
        toggleViewMode();
        break;

      // Undo
      case 'z':
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault();
          handleUndo();
        }
        break;

      // Delete = Buang
      case 'Delete':
        if (item && projectId) {
          setCategoryForItem(item.id, projectId, 1);
          flashCategory(1);
          toast.success('Buang', 1500);
          navigateNext();
        }
        break;

      // Star ratings (0-5)
      case '0': case '5': case '6': case '7': case '8': case '9':
        // 5-9 could be star ratings, but we only use 0-5
        if (item && parseInt(e.key) <= 5) {
          setStarRatingForItem(item.id, parseInt(e.key));
        }
        break;

      // Show shortcuts
      case '?':
        showShortcuts = !showShortcuts;
        break;

      // Toggle metadata panel
      case 'i':
        showMetadata = !showMetadata;
        break;

      // Go home
      case 'Escape':
        if (showShortcuts) {
          showShortcuts = false;
        } else if ($viewMode === 'preview') {
          viewMode.set('grid');
        }
        break;
    }
  }

  async function handleUndo() {
    if (!$currentProject) return;
    try {
      const result = await bridge.undoLastAction($currentProject.id);
      if (result) {
        toast.info('Undone');
        await loadMediaItems($currentProject.id);
        await loadCategoryStats($currentProject.id);
      } else {
        toast.info('Nothing to undo');
      }
    } catch {
      toast.error('Undo failed');
    }
  }

  function flashCategory(catId: number) {
    categoryFlash = catId;
    setTimeout(() => { categoryFlash = null; }, 300);
  }

  function handleGridSelect(index: number) {
    navigateTo(index);
  }

  function handleGridDoubleClick(index: number) {
    navigateTo(index);
    viewMode.set('preview');
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="cull-workspace">
  <!-- Sidebar -->
  <Sidebar />

  <!-- Main content area -->
  <div class="main-area">
    <!-- Toolbar -->
    <Toolbar
      bind:thumbnailSize={thumbnailSize}
      bind:showMetadata={showMetadata}
      onShowShortcuts={() => showShortcuts = true}
      onGoHome={() => goto('/')}
    />

    <!-- Scan progress overlay -->
    {#if $isScanning}
      <ProgressBar />
    {/if}

    <!-- Category flash indicator -->
    {#if categoryFlash}
      <div
        class="category-flash"
        style="background-color: {getCategoryColor(categoryFlash)}"
      >
        <span>{getCategoryName(categoryFlash)}</span>
      </div>
    {/if}

    <!-- Content area -->
    <div class="content-area">
      <div class="media-container">
        {#if $viewMode === 'grid'}
          <GridView
            items={$mediaItems}
            selectedIndex={$selectedIndex}
            {thumbnailSize}
            onSelect={handleGridSelect}
            onDoubleClick={handleGridDoubleClick}
          />
        {:else}
          <PreviewView
            item={$currentItem}
            items={$mediaItems}
            selectedIndex={$selectedIndex}
            onNavigate={navigateTo}
          />
        {/if}

        {#if !$isLoading && $mediaItems.length === 0}
          <div class="empty-media">
            <p>No media items found</p>
            <p class="empty-sub">
              {#if $categoryFilter || $uncategorizedOnly}
                Try clearing filters
              {:else}
                Open a folder to start culling
              {/if}
            </p>
          </div>
        {/if}
      </div>

      <!-- Metadata panel -->
      {#if showMetadata}
        <MetadataPanel
          {exifData}
          item={$currentItem}
          onClose={() => showMetadata = false}
        />
      {/if}
    </div>

    <!-- Status bar -->
    <div class="status-bar">
      <span class="status-item">
        {$selectedIndex + 1} / {$mediaItems.length}
        {#if $totalItems > $mediaItems.length}
          <span class="text-dim">(of {$totalItems})</span>
        {/if}
      </span>
      {#if $currentItem}
        <span class="status-item truncate">{$currentItem.file_name}</span>
        <span class="status-item">{formatFileSize($currentItem.file_size)}</span>
        {#if $currentItem.category_id}
          <span
            class="status-category"
            style="color: {getCategoryColor($currentItem.category_id)}"
          >
            {getCategoryName($currentItem.category_id)}
          </span>
        {/if}
      {/if}
      <span class="status-item status-right">
        <kbd>?</kbd> Shortcuts
      </span>
    </div>
  </div>
</div>

<!-- Keyboard shortcuts modal -->
{#if showShortcuts}
  <KeyboardShortcutsModal onClose={() => showShortcuts = false} />
{/if}

<style>
  .cull-workspace {
    display: flex;
    width: 100%;
    height: 100vh;
    overflow: hidden;
    background: var(--bg-primary);
  }

  .main-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
  }

  .content-area {
    flex: 1;
    display: flex;
    overflow: hidden;
    position: relative;
  }

  .media-container {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .empty-media {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    color: var(--text-tertiary);
    font-size: var(--text-md);
  }

  .empty-sub {
    font-size: var(--text-sm);
    opacity: 0.6;
  }

  /* Status bar */
  .status-bar {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-1) var(--space-4);
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-subtle);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    min-height: 24px;
    flex-shrink: 0;
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  .status-category {
    font-weight: 600;
  }

  .status-right {
    margin-left: auto;
  }

  .text-dim {
    opacity: 0.5;
  }

  .status-bar kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 16px;
    height: 16px;
    padding: 0 3px;
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    color: var(--text-tertiary);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    border-radius: 3px;
  }

  /* Category flash */
  .category-flash {
    position: absolute;
    top: var(--toolbar-height);
    left: 50%;
    transform: translateX(-50%);
    z-index: 100;
    padding: var(--space-2) var(--space-6);
    border-radius: var(--radius-full);
    color: white;
    font-weight: 700;
    font-size: var(--text-md);
    animation: categoryFlash 0.3s ease-out forwards;
    pointer-events: none;
    opacity: 0.9;
    box-shadow: var(--shadow-lg);
  }
</style>
