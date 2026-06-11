<script lang="ts">
  import {
    categoryStats, categories, categoryFilter, uncategorizedOnly, totalItems,
    setFilterCategory, setFilterUncategorized, clearFilters,
  } from '$lib/stores/media';
  import { currentProject } from '$lib/stores/project';
  import { getCategoryColor } from '$lib/types';
  import { goto } from '$app/navigation';

  function handleFilterAll() {
    clearFilters();
  }
</script>

<aside class="sidebar">
  <!-- Logo / back -->
  <div class="sidebar-header">
    <button class="btn btn-ghost sidebar-back" onclick={() => goto('/')}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="15 18 9 12 15 6"/>
      </svg>
    </button>
    <h2 class="sidebar-title truncate">{$currentProject?.name ?? 'Keepix'}</h2>
  </div>

  <!-- Category filters -->
  <nav class="sidebar-nav">
    <div class="nav-section-title">Categories</div>

    <!-- All items -->
    <button
      class="nav-item"
      class:active={!$categoryFilter && !$uncategorizedOnly}
      onclick={handleFilterAll}
    >
      <span class="nav-dot" style="background: var(--text-tertiary)"></span>
      <span class="nav-label">All</span>
      <span class="nav-count">{$totalItems}</span>
    </button>

    <!-- Uncategorized -->
    {#each $categoryStats as stat (stat.category_id ?? 'uncat')}
      {#if stat.category_id === null}
        <button
          class="nav-item"
          class:active={$uncategorizedOnly}
          onclick={setFilterUncategorized}
        >
          <span class="nav-dot" style="background: var(--text-tertiary); opacity: 0.4"></span>
          <span class="nav-label">Uncategorized</span>
          <span class="nav-count">{stat.count}</span>
        </button>
      {/if}
    {/each}

    <div class="nav-divider"></div>

    <!-- Category items -->
    {#each $categoryStats as stat (stat.category_id ?? 'uncat')}
      {#if stat.category_id !== null}
        <button
          class="nav-item"
          class:active={$categoryFilter === stat.category_id}
          onclick={() => setFilterCategory(stat.category_id!)}
        >
          <span class="nav-dot" style="background: {getCategoryColor(stat.category_id)}"></span>
          <span class="nav-label">{stat.category_name}</span>
          <span class="nav-count">{stat.count}</span>
          <kbd class="nav-hotkey">{stat.category_id}</kbd>
        </button>
      {/if}
    {/each}
  </nav>

  <!-- Keyboard hints -->
  <div class="sidebar-footer">
    <div class="hint-row">
      <kbd>←</kbd><kbd>→</kbd> <span>Navigate</span>
    </div>
    <div class="hint-row">
      <kbd>Space</kbd> <span>Toggle view</span>
    </div>
    <div class="hint-row">
      <kbd>Ctrl+Z</kbd> <span>Undo</span>
    </div>
  </div>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    height: 100%;
    background: rgba(17, 17, 20, 0.4);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-right: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    position: relative;
    z-index: 10;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-3);
    border-bottom: 1px solid var(--border-subtle);
    min-height: var(--toolbar-height);
  }

  .sidebar-back {
    padding: var(--space-1);
    min-width: 28px;
    min-height: 28px;
    color: var(--text-tertiary);
  }

  .sidebar-title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-primary);
  }

  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-2);
  }

  .nav-section-title {
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.8px;
    padding: var(--space-2) var(--space-2) var(--space-1);
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-2);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    font-size: var(--text-sm);
    color: var(--text-secondary);
    background: none;
    border: none;
    text-align: left;
    font-family: var(--font-sans);
  }

  .nav-item:hover {
    background: var(--surface-card-hover);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: var(--accent-soft);
    color: var(--text-primary);
  }

  .nav-dot {
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full);
    flex-shrink: 0;
  }

  .nav-label {
    flex: 1;
    text-align: left;
  }

  .nav-count {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    min-width: 20px;
    text-align: right;
  }

  .nav-hotkey {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 4px;
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    color: var(--text-tertiary);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    border-radius: 3px;
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  .nav-item:hover .nav-hotkey {
    opacity: 1;
  }

  .nav-divider {
    height: 1px;
    background: var(--border-subtle);
    margin: var(--space-2) 0;
  }

  .sidebar-footer {
    padding: var(--space-3);
    border-top: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .hint-row {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
  }

  .hint-row span {
    margin-left: var(--space-1);
  }

  .sidebar-footer kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 4px;
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    color: var(--text-tertiary);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    border-radius: 3px;
  }
</style>
