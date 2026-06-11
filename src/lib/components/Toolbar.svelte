<script lang="ts">
  import { viewMode, toggleViewMode } from '$lib/stores/media';

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
</script>

<div class="toolbar">
  <div class="toolbar-left">
    <!-- View mode toggle -->
    <div class="view-toggle">
      <button
        class="btn btn-icon btn-ghost"
        class:active={$viewMode === 'grid'}
        onclick={() => viewMode.set('grid')}
        data-tooltip="Grid view"
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
        data-tooltip="Preview mode"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="2"/>
          <line x1="3" y1="15" x2="21" y2="15"/>
        </svg>
      </button>
    </div>

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
    <!-- Toggle metadata panel -->
    <button
      class="btn btn-icon btn-ghost"
      class:active={showMetadata}
      onclick={() => showMetadata = !showMetadata}
      data-tooltip="Info panel (I)"
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
      data-tooltip="Shortcuts (?)"
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
</style>
