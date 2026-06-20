<script lang="ts">
  import { viewMode, toggleViewMode, sortBy, sortOrder, compareMode, syncZoom, loadMediaItems, diagnosticsMode, showHistogram, autoAdvance } from '$lib/stores/media';
  import { currentProject } from '$lib/stores/project';
  import { t, locale } from '$lib/i18n';

  let {
    thumbnailSize = $bindable(200),
    showMetadata = $bindable(false),
    onShowShortcuts,
    onGoHome,
  }: {
    thumbnailSize: number;
    showMetadata: boolean;
    onShowShortcuts: () => void;
    onGoHome: () => void;
  } = $props();

  async function handleSortChange() {
    if ($currentProject) {
      await loadMediaItems($currentProject.id);
    }
  }

  async function toggleSortOrder() {
    sortOrder.update(o => o === 'asc' ? 'desc' : 'asc');
    if ($currentProject) {
      await loadMediaItems($currentProject.id);
    }
  }
</script>

<div class="toolbar">
  <div class="toolbar-left">
    <!-- View mode toggle -->
    <div class="view-toggle">
      <button
        class="btn btn-icon btn-ghost"
        class:active={$viewMode === 'grid'}
        onclick={() => viewMode.set('grid')}
        data-tooltip={$t('toolbar.view.grid')}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <rect x="3" y="3" width="7" height="7" rx="1"/>
          <rect x="14" y="3" width="7" height="7" rx="1"/>
          <rect x="3" y="14" width="7" height="7" rx="1"/>
          <rect x="14" y="14" width="7" height="7" rx="1"/>
        </svg>
      </button>
      <button
        class="btn btn-icon btn-ghost"
        class:active={$viewMode === 'preview'}
        onclick={() => viewMode.set('preview')}
        data-tooltip={$t('toolbar.view.preview')}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="2"/>
          <line x1="3" y1="15" x2="21" y2="15"/>
        </svg>
      </button>
    </div>

    <!-- Sort controls -->
    <div class="toolbar-divider"></div>
    <span class="toolbar-label">{$t('toolbar.sort')}:</span>
    <select
      class="toolbar-select"
      value={$sortBy}
      onchange={(e) => { sortBy.set((e.currentTarget as HTMLSelectElement).value as any); handleSortChange(); }}
    >
      <option value="name">{$t('toolbar.sort.name')}</option>
      <option value="date">{$t('toolbar.sort.date')}</option>
      <option value="size">{$t('toolbar.sort.size')}</option>
      <option value="rating">{$t('toolbar.sort.rating')}</option>
      <option value="type">{$t('toolbar.sort.type')}</option>
    </select>

    <button
      class="btn btn-icon btn-ghost sort-order-btn"
      onclick={toggleSortOrder}
      data-tooltip={$t($sortOrder === 'asc' ? 'toolbar.sort.asc' : 'toolbar.sort.desc')}
    >
      {#if $sortOrder === 'asc'}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <polyline points="19 12 12 5 5 12"/>
        </svg>
      {:else}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <polyline points="19 12 12 19 5 12"/>
        </svg>
      {/if}
    </button>

    <!-- Compare Mode options (Preview mode only) -->
    {#if $viewMode === 'preview'}
      <div class="toolbar-divider"></div>
      <span class="toolbar-label">{$t('toolbar.compare')}:</span>
      <select
        class="toolbar-select"
        value={$compareMode}
        onchange={(e) => compareMode.set((e.currentTarget as HTMLSelectElement).value as any)}
      >
        <option value="single">{$t('toolbar.compare.single')}</option>
        <option value="2-up">{$t('toolbar.compare.2up')}</option>
        <option value="4-up">{$t('toolbar.compare.4up')}</option>
      </select>

      {#if $compareMode !== 'single'}
        <button
          class="btn btn-icon btn-ghost compare-sync-btn"
          class:active={$syncZoom}
          onclick={() => syncZoom.update(z => !z)}
          data-tooltip={$t('toolbar.compare.sync')}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M15 3h6v6"/>
            <path d="M9 21H3v-6"/>
            <line x1="21" y1="3" x2="14" y2="10"/>
            <line x1="3" y1="21" x2="10" y2="14"/>
          </svg>
        </button>
      {/if}

      <div class="toolbar-divider"></div>
      <span class="toolbar-label">{$t('toolbar.diagnostics')}:</span>
      <select
        class="toolbar-select"
        value={$diagnosticsMode}
        onchange={(e) => diagnosticsMode.set((e.currentTarget as HTMLSelectElement).value as any)}
      >
        <option value="none">{$t('toolbar.diagnostics.none')}</option>
        <option value="peaking">{$t('toolbar.diagnostics.peaking')}</option>
        <option value="zebra">{$t('toolbar.diagnostics.zebra')}</option>
      </select>
    {/if}

    <!-- Thumbnail size slider (grid mode only) -->
    {#if $viewMode === 'grid'}
      <div class="thumb-slider">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" opacity="0.4">
          <rect x="5" y="5" width="6" height="6" rx="1"/>
          <rect x="13" y="5" width="6" height="6" rx="1"/>
          <rect x="5" y="13" width="6" height="6" rx="1"/>
          <rect x="13" y="13" width="6" height="6" rx="1"/>
        </svg>
        <input
          type="range"
          min="100"
          max="400"
          step="20"
          bind:value={thumbnailSize}
          class="slider"
        />
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" opacity="0.4">
          <rect x="3" y="3" width="8" height="8" rx="1"/>
          <rect x="13" y="3" width="8" height="8" rx="1"/>
          <rect x="3" y="13" width="8" height="8" rx="1"/>
          <rect x="13" y="13" width="8" height="8" rx="1"/>
        </svg>
      </div>
    {/if}
  </div>

  <div class="toolbar-right">
    <!-- Toggle Live Histogram -->
    {#if $viewMode === 'preview'}
      <button
        class="btn btn-icon btn-ghost"
        class:active={$showHistogram}
        onclick={() => showHistogram.update(h => !h)}
        data-tooltip={$t('toolbar.histogram.tooltip')}
        aria-label="Toggle Histogram"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 20V10M12 20V4M6 20v-6"/>
        </svg>
      </button>
    {/if}

    <!-- Toggle Auto-Advance -->
    <button
      class="btn btn-icon btn-ghost"
      class:active={$autoAdvance}
      onclick={() => autoAdvance.update(a => !a)}
      data-tooltip={$t('toolbar.autoadvance.tooltip')}
      aria-label="Toggle Auto-Advance"
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
      </svg>
    </button>

    <!-- Toggle metadata panel -->
    <button
      class="btn btn-icon btn-ghost"
      class:active={showMetadata}
      onclick={() => showMetadata = !showMetadata}
      data-tooltip={$t('toolbar.info.tooltip')}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="16" x2="12" y2="12"/>
        <line x1="12" y1="8" x2="12.01" y2="8"/>
      </svg>
    </button>

    <!-- Keyboard shortcuts -->
    <button
      class="btn btn-icon btn-ghost"
      onclick={onShowShortcuts}
      data-tooltip={$t('toolbar.shortcuts.tooltip')}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="2" y="4" width="20" height="16" rx="2"/>
        <line x1="6" y1="8" x2="6" y2="8"/>
        <line x1="10" y1="8" x2="10" y2="8"/>
        <line x1="14" y1="8" x2="14" y2="8"/>
        <line x1="18" y1="8" x2="18" y2="8"/>
        <line x1="8" y1="16" x2="16" y2="16"/>
        <line x1="6" y1="12" x2="6" y2="12"/>
        <line x1="10" y1="12" x2="10" y2="12"/>
        <line x1="14" y1="12" x2="14" y2="12"/>
        <line x1="18" y1="12" x2="18" y2="12"/>
      </svg>
    </button>
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-3);
    height: var(--toolbar-height);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
    gap: var(--space-4);
  }

  .toolbar-left, .toolbar-right {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .view-toggle {
    display: flex;
    background: var(--bg-tertiary);
    border-radius: var(--radius-md);
    padding: 2px;
    gap: 2px;
  }

  .view-toggle .btn-icon {
    border-radius: var(--radius-sm);
    min-width: 28px;
    min-height: 28px;
    padding: 4px;
  }

  .view-toggle .btn-icon.active,
  .toolbar-right .btn-icon.active {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .thumb-slider {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-left: var(--space-2);
  }

  .slider {
    width: 100px;
    height: 4px;
    appearance: none;
    background: var(--bg-tertiary);
    border-radius: var(--radius-full);
    outline: none;
    cursor: pointer;
  }

  .slider::-webkit-slider-thumb {
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: var(--radius-full);
    background: var(--accent);
    cursor: pointer;
    transition: transform var(--transition-fast);
  }

  .slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
  }

  .toolbar-select {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    padding: 2px var(--space-2);
    border-radius: var(--radius-md);
    font-size: var(--text-xs);
    outline: none;
    cursor: pointer;
    transition: all var(--transition-fast);
    height: 28px;
  }

  .toolbar-select:hover {
    border-color: var(--border-strong);
    color: var(--text-primary);
  }

  .toolbar-divider {
    width: 1px;
    height: 16px;
    background: var(--border-subtle);
    margin: 0 var(--space-1);
  }

  .toolbar-label {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 600;
  }

  .sort-order-btn, .compare-sync-btn {
    min-width: 28px;
    min-height: 28px;
    padding: 4px;
    border-radius: var(--radius-md);
  }

  .compare-sync-btn.active {
    background: var(--accent-soft);
    color: var(--accent);
  }
</style>
