<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { currentProject } from '$lib/stores/project';
  import { categories, categoryStats, loadCategoryStats } from '$lib/stores/media';
  import { toast } from '$lib/stores/toast';
  import * as bridge from '$lib/services/tauri-bridge';
  import type { ExportProgress } from '$lib/types';

  let {
    onClose,
  }: {
    onClose: () => void;
  } = $props();

  let targetFolder = $state('');
  let selectedCategories = $state<number[]>([2]); // Default: Simpan (2)
  let exportUncategorized = $state(false);
  let actionType = $state<'copy' | 'move' | 'list'>('copy');
  let conflictBehavior = $state<'overwrite' | 'skip' | 'rename'>('rename');

  let isExporting = $state(false);
  let progress = $state<ExportProgress | null>(null);
  let unlistenExport: (() => void) | null = null;

  onMount(async () => {
    // Set default target folder inside project's parent directory + /Exported
    if ($currentProject) {
      const rootPath = $currentProject.root_path.replace(/\\/g, '/');
      targetFolder = `${rootPath}/Keepix_Exported`;
    }
  });

  onDestroy(() => {
    if (unlistenExport) {
      unlistenExport();
    }
  });

  async function handleSelectFolder() {
    try {
      const folder = await bridge.pickFolder();
      if (folder) {
        targetFolder = folder;
      }
    } catch (err) {
      console.error(err);
      toast.error('Failed to pick folder');
    }
  }

  async function handleStartExport() {
    if (!targetFolder.trim()) {
      toast.error('Please select a target folder');
      return;
    }

    if (selectedCategories.length === 0 && !exportUncategorized) {
      toast.error('Please select at least one category to export');
      return;
    }

    if (!$currentProject) return;

    isExporting = true;
    progress = {
      phase: 'exporting',
      current: 0,
      total: 0,
      current_file: 'Initializing...',
      status: 'Connecting...',
    };

    try {
      // Listen to progress events
      unlistenExport = await bridge.onExportProgress((event) => {
        progress = event;
        if (event.phase === 'complete') {
          isExporting = false;
          toast.success('Export completed successfully!');
          if ($currentProject) {
            loadCategoryStats($currentProject.id);
          }
          if (unlistenExport) {
            unlistenExport();
            unlistenExport = null;
          }
        }
      });

      // Trigger the export
      const count = await bridge.exportMediaItems(
        $currentProject.id,
        targetFolder,
        selectedCategories,
        exportUncategorized,
        actionType,
        conflictBehavior
      );

      if (count === 0) {
        isExporting = false;
        toast.info('No files match the selected categories.');
        if (unlistenExport) {
          unlistenExport();
          unlistenExport = null;
        }
      }
    } catch (err) {
      isExporting = false;
      progress = null;
      console.error(err);
      toast.error('Export failed: ' + err);
      if (unlistenExport) {
        unlistenExport();
        unlistenExport = null;
      }
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget && !isExporting) {
      onClose();
    }
  }

  function toggleCategory(catId: number) {
    if (selectedCategories.includes(catId)) {
      selectedCategories = selectedCategories.filter(id => id !== catId);
    } else {
      selectedCategories = [...selectedCategories, catId];
    }
  }
</script>

<div class="modal-overlay" onclick={handleOverlayClick} role="dialog" aria-labelledby="export-title">
  <div class="modal-content export-modal glass-card">
    <div class="modal-header">
      <h2 id="export-title" class="modal-title">Export Culled Media</h2>
      <button class="btn btn-ghost btn-icon close-btn" onclick={onClose} disabled={isExporting} aria-label="Close export dialog">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    {#if !isExporting && !progress}
      <div class="modal-body">
        <!-- Folder picker -->
        <div class="form-group">
          <label class="form-label" for="target-dir-input">Target Destination</label>
          <div class="folder-picker-row">
            <input
              id="target-dir-input"
              type="text"
              bind:value={targetFolder}
              class="form-input"
              placeholder="Select destination folder"
            />
            <button class="btn btn-secondary" onclick={handleSelectFolder}>
              Browse...
            </button>
          </div>
        </div>

        <!-- Categories filters -->
        <div class="form-group">
          <span class="form-label">Export Categories</span>
          <div class="categories-checkbox-grid">
            {#each $categories as cat}
              <button
                class="checkbox-chip"
                class:active={selectedCategories.includes(cat.id)}
                onclick={() => toggleCategory(cat.id)}
              >
                <span class="chip-dot" style="background: {cat.color}"></span>
                <span class="chip-label">{cat.name}</span>
              </button>
            {/each}
            <button
              class="checkbox-chip"
              class:active={exportUncategorized}
              onclick={() => exportUncategorized = !exportUncategorized}
            >
              <span class="chip-dot" style="background: var(--text-tertiary); opacity: 0.5;"></span>
              <span class="chip-label">Uncategorized</span>
            </button>
          </div>
        </div>

        <div class="export-settings-row">
          <!-- Action options -->
          <div class="form-group flex-1">
            <label class="form-label" for="action-select">Export Action</label>
            <select id="action-select" class="form-select" bind:value={actionType}>
              <option value="copy">Copy files (Safest)</option>
              <option value="move">Move files (Warning: Deletes source!)</option>
              <option value="list">Generate file paths list (.txt only)</option>
            </select>
          </div>

          <!-- Conflict resolution -->
          {#if actionType !== 'list'}
            <div class="form-group flex-1">
              <label class="form-label" for="conflict-select">If File Exists</label>
              <select id="conflict-select" class="form-select" bind:value={conflictBehavior}>
                <option value="rename">Rename automatically (Save both)</option>
                <option value="overwrite">Overwrite existing</option>
                <option value="skip">Skip files</option>
              </select>
            </div>
          {/if}
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-ghost" onclick={onClose}>Cancel</button>
        <button class="btn btn-primary start-export-btn" onclick={handleStartExport}>
          Start Export
        </button>
      </div>
    {:else}
      <!-- Export Progress phase -->
      <div class="modal-body progress-body">
        <div class="progress-info">
          <span class="progress-status">
            {#if progress?.phase === 'complete'}
              ✨ Export Complete!
            {:else}
              ⚙️ Exporting files...
            {/if}
          </span>
          {#if progress && progress.total > 0}
            <span class="progress-counts">
              {progress.current} / {progress.total}
            </span>
          {/if}
        </div>

        <div class="progress-bar">
          <div
            class="progress-bar-fill"
            style="width: {progress && progress.total > 0
              ? (progress.current / progress.total) * 100
              : 0}%"
          ></div>
        </div>

        <div class="progress-log">
          <p class="progress-filename truncate">{progress?.current_file || 'Waiting...'}</p>
          <p class="progress-details">{progress?.status || ''}</p>
        </div>

        {#if progress?.phase === 'complete'}
          <div class="modal-footer" style="padding-top: var(--space-4); margin-top: var(--space-4); border-top: 1px solid var(--border-subtle)">
            <button class="btn btn-primary w-full" style="width: 100%;" onclick={onClose}>
              Close Dialog
            </button>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .export-modal {
    max-width: 520px;
    width: 90%;
    padding: var(--space-6);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-5);
  }

  .modal-title {
    font-size: var(--text-md);
    font-weight: 700;
    color: var(--text-primary);
  }

  .modal-body {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .form-label {
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .folder-picker-row {
    display: flex;
    gap: var(--space-2);
  }

  .form-input {
    flex: 1;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0 var(--space-3);
    color: var(--text-primary);
    font-size: var(--text-sm);
    outline: none;
    height: 36px;
  }

  .form-input:focus {
    border-color: var(--accent);
  }

  .categories-checkbox-grid {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-top: 4px;
  }

  .checkbox-chip {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 6px var(--space-3);
    border-radius: var(--radius-full);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: var(--text-xs);
    transition: all var(--transition-fast);
  }

  .checkbox-chip:hover {
    border-color: var(--border-strong);
    color: var(--text-primary);
  }

  .checkbox-chip.active {
    background: var(--accent-soft);
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .chip-dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
  }

  .export-settings-row {
    display: flex;
    gap: var(--space-4);
  }

  .flex-1 {
    flex: 1;
  }

  .form-select {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0 var(--space-2);
    color: var(--text-primary);
    font-size: var(--text-xs);
    outline: none;
    height: 36px;
    cursor: pointer;
  }

  .form-select:focus {
    border-color: var(--accent);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    margin-top: var(--space-6);
  }

  /* Progress body styles */
  .progress-body {
    padding: var(--space-4) 0;
    gap: var(--space-3);
  }

  .progress-info {
    display: flex;
    justify-content: space-between;
    font-size: var(--text-sm);
    font-weight: 600;
  }

  .progress-status {
    color: var(--text-primary);
  }

  .progress-counts {
    font-family: var(--font-mono);
    color: var(--text-tertiary);
  }

  .progress-log {
    background: var(--bg-tertiary);
    padding: var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .progress-filename {
    font-size: var(--text-xs);
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }

  .progress-details {
    font-size: 10px;
    color: var(--text-tertiary);
    margin-top: 2px;
  }
</style>
