<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { currentProject, scanProgress, isScanning } from '$lib/stores/project';
  import {
    mediaItems, totalItems, selectedIndex, viewMode, isLoading,
    currentItem, categoryFilter, uncategorizedOnly, categoryStats,
    loadMediaItems, loadCategoryStats, loadCategories, loadExifFilters,
    setCategoryForItem, setStarRatingForItem, setColorLabelForItem,
    navigateNext, navigatePrev, navigateTo, toggleViewMode,
    setFilterCategory, setFilterUncategorized, clearFilters,
    categories, compareMode, syncZoom, diagnosticsMode, showHistogram, autoAdvance, displayItems,
  } from '$lib/stores/media';
  import { toast } from '$lib/stores/toast';
  import { toAssetUrl } from '$lib/services/tauri-bridge';
  import * as bridge from '$lib/services/tauri-bridge';
  import { getCategoryName, getCategoryColor, formatFileSize, type ExifData, type MediaItem } from '$lib/types';
  import GridView from '$lib/components/GridView.svelte';
  import PreviewView from '$lib/components/PreviewView.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import Toolbar from '$lib/components/Toolbar.svelte';
  import MetadataPanel from '$lib/components/MetadataPanel.svelte';
  import ProgressBar from '$lib/components/ProgressBar.svelte';
  import KeyboardShortcutsModal from '$lib/components/KeyboardShortcutsModal.svelte';
  import ExportModal from '$lib/components/ExportModal.svelte';
  import SplitPane from '$lib/components/SplitPane.svelte';
  import EditPanel from '$lib/components/EditPanel.svelte';

  let showMetadata = $state(false);
  let showShortcuts = $state(false);
  let showExport = $state(false);
  let exifData = $state<ExifData | null>(null);
  let thumbnailSize = $state(200);
  let categoryFlash = $state<number | null>(null);

  // ... (rest of logic up to UI) ...
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

  let scanUnlisten: (() => void) | null = null;

  onMount(async () => {
    if (!$currentProject) {
      goto('/');
      return;
    }

    // Load EXIF filters
    await loadExifFilters($currentProject.id);

    // If items aren't loaded yet, load them
    if ($mediaItems.length === 0) {
      await loadMediaItems($currentProject.id);
      await loadCategoryStats($currentProject.id);
    }

    // Set up real-time listener for scan progress to load thumbnails dynamically!
    let lastReloadTime = 0;
    let reloadTimeout: any = null;

    scanUnlisten = await bridge.onScanProgress(async (progress) => {
      if (progress.phase === 'thumbnails') {
        const now = Date.now();
        if (now - lastReloadTime > 1500) {
          lastReloadTime = now;
          await loadMediaItems($currentProject!.id);
          await loadCategoryStats($currentProject!.id);
        } else {
          clearTimeout(reloadTimeout);
          reloadTimeout = setTimeout(async () => {
            if ($currentProject) {
              await loadMediaItems($currentProject.id);
              await loadCategoryStats($currentProject.id);
            }
          }, 1500);
        }
      } else if (progress.phase === 'complete') {
        clearTimeout(reloadTimeout);
        if ($currentProject) {
          await loadMediaItems($currentProject.id);
          await loadCategoryStats($currentProject.id);
          await loadExifFilters($currentProject.id);
        }
      }
    });
  });

  onDestroy(() => {
    if (scanUnlisten) {
      scanUnlisten();
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
        if ($autoAdvance) {
          navigateNext();
        }
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

      case 'c':
      case 'C':
        if ($viewMode === 'preview') {
          compareMode.update(m => {
            if (m === 'single') return '2-up';
            if (m === '2-up') return '4-up';
            return 'single';
          });
          toast.info(`Compare Mode: ${$compareMode.toUpperCase()}`);
        }
        break;

      // Undo & Sync Zoom Toggle
      case 'z':
      case 'Z':
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault();
          handleUndo();
        } else if ($viewMode === 'preview' && $compareMode !== 'single') {
          syncZoom.update(sz => !sz);
          toast.info($syncZoom ? 'Zoom linked' : 'Zoom unlinked');
        }
        break;

      // Delete = Buang
      case 'Delete':
        if (item && projectId) {
          setCategoryForItem(item.id, projectId, 1);
          flashCategory(1);
          toast.success('Buang', 1500);
          if ($autoAdvance) {
            navigateNext();
          }
        }
        break;

      // Star ratings (0 or 5)
      case '0':
      case '5':
        if (item) {
          const rating = parseInt(e.key);
          setStarRatingForItem(item.id, rating);
          toast.success(`Rating: ${rating} Stars`, 1500);
          if ($autoAdvance) {
            navigateNext();
          }
        }
        break;

      // Color labels (6-9)
      case '6':
      case '7':
      case '8':
      case '9': {
        if (!item) return;
        let color: string | null = null;
        let labelName = 'None';
        if (e.key === '6') { color = 'red'; labelName = 'Red'; }
        else if (e.key === '7') { color = 'orange'; labelName = 'Orange'; }
        else if (e.key === '8') { color = 'yellow'; labelName = 'Yellow'; }
        else if (e.key === '9') { color = 'green'; labelName = 'Green'; }

        setColorLabelForItem(item.id, color);
        toast.success(`Label: ${labelName}`, 1500);
        if ($autoAdvance) {
          navigateNext();
        }
        break;
      }

      // Toggle histogram
      case 'h':
      case 'H':
        showHistogram.update(h => !h);
        toast.info($showHistogram ? 'Histogram visible' : 'Histogram hidden');
        break;

      // Cycle diagnostics overlay
      case 'o':
      case 'O':
        diagnosticsMode.update(mode => {
          if (mode === 'none') return 'peaking';
          if (mode === 'peaking') return 'zebra';
          return 'none';
        });
        toast.info(`Diagnostics: ${$diagnosticsMode.toUpperCase()}`);
        break;

      // Toggle auto-advance
      case 'a':
      case 'A':
        autoAdvance.update(a => !a);
        toast.info($autoAdvance ? 'Auto-Advance ON' : 'Auto-Advance OFF');
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

  function handleGridDoubleClick(index: number, burstItems?: MediaItem[]) {
    navigateTo(index);
    if (burstItems && burstItems.length > 1) {
      if (burstItems.length === 2) {
        compareMode.set('2-up');
      } else {
        compareMode.set('4-up');
      }
      toast.success(`Comparing ${burstItems.length} burst photos`);
    } else {
      compareMode.set('single');
    }
    viewMode.set('preview');
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="cull-workspace">
  <SplitPane minSizes={[240, 400, 260]} defaultSizes={[280, 800, 320]}>
    
    <!-- LEFT PANE: Sidebar -->
    <div slot="left" style="height: 100%;">
      <Sidebar onOpenExport={() => showExport = true} />
    </div>

    <!-- CENTER PANE: Main Content -->
    <div slot="center" class="main-area">
      <Toolbar
        bind:thumbnailSize={thumbnailSize}
        bind:showMetadata={showMetadata}
        onShowShortcuts={() => showShortcuts = true}
        onGoHome={() => goto('/')}
      />

      {#if $isScanning}
        <ProgressBar />
      {/if}

      {#if categoryFlash}
        <div class="category-flash" style="background-color: {getCategoryColor(categoryFlash)}">
          <span>{getCategoryName(categoryFlash)}</span>
        </div>
      {/if}

      <div class="content-area">
        <div class="media-container">
          {#if $viewMode === 'grid'}
            <GridView
              items={$displayItems}
              originalItems={$mediaItems}
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

        <!-- Metadata panel overlays the center pane -->
        {#if showMetadata}
          <MetadataPanel
            {exifData}
            item={$currentItem}
            onClose={() => showMetadata = false}
          />
        {/if}
      </div>

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
            <span class="status-category" style="color: {getCategoryColor($currentItem.category_id)}">
              {getCategoryName($currentItem.category_id)}
            </span>
          {/if}
        {/if}
        <span class="status-item status-right">
          <kbd>?</kbd> Shortcuts
        </span>
      </div>
    </div>

    <!-- RIGHT PANE: Edit Panel -->
    <div slot="right" style="height: 100%;">
      <EditPanel item={$currentItem} />
    </div>

  </SplitPane>
</div>

<!-- Keyboard shortcuts modal -->
{#if showShortcuts}
  <KeyboardShortcutsModal onClose={() => showShortcuts = false} />
{/if}

<!-- Export modal -->
{#if showExport}
  <ExportModal onClose={() => showExport = false} />
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
