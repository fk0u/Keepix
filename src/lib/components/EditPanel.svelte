<script lang="ts">
  import { t } from '$lib/i18n';
  import { invoke } from '@tauri-apps/api/core';
  import type { MediaItem } from '$lib/types';
  import { currentItem, displayItems, mediaItems } from '$lib/stores/media';
  import { toast } from '$lib/stores/toast';
  import { get } from 'svelte/store';

  let { item }: { item: MediaItem | null } = $props();

  // Edit State
  let adjustments = $state({
    exposure: 0,
    contrast: 0,
    highlights: 0,
    shadows: 0,
    temperature: 0,
    tint: 0,
    saturation: 0,
    vibrance: 0,
  });

  let activePreset = $state<string | null>(null);

  // Load adjustments when item changes
  let prevItemId = '';
  $effect(() => {
    if (item && item.id !== prevItemId) {
      prevItemId = item.id;
      if (item.adjustments) {
        try {
          adjustments = JSON.parse(item.adjustments);
        } catch {
          resetAdjustments();
        }
      } else {
        resetAdjustments();
      }
      activePreset = item.applied_preset || null;
    }
  });

  function resetAdjustments() {
    adjustments = {
      exposure: 0,
      contrast: 0,
      highlights: 0,
      shadows: 0,
      temperature: 0,
      tint: 0,
      saturation: 0,
      vibrance: 0,
    };
  }

  // Presets definition
  const presets = [
    { id: 'none', label: 'edit.preset.none', values: null },
    { id: 'cinematic', label: 'edit.preset.cinematic', values: { exposure: -10, contrast: 20, highlights: -15, shadows: 10, temperature: -5, tint: 5, saturation: -15, vibrance: 10 } },
    { id: 'vintage', label: 'edit.preset.vintage', values: { exposure: 10, contrast: -15, highlights: -20, shadows: 25, temperature: 15, tint: -5, saturation: -20, vibrance: -10 } },
    { id: 'bw', label: 'edit.preset.bw', values: { exposure: 0, contrast: 30, highlights: 20, shadows: -20, temperature: 0, tint: 0, saturation: -100, vibrance: 0 } },
    { id: 'punchy', label: 'edit.preset.punchy', values: { exposure: 5, contrast: 15, highlights: -10, shadows: 5, temperature: 0, tint: 0, saturation: 10, vibrance: 25 } }
  ];

  async function applyPreset(presetId: string) {
    if (!item) return;
    
    activePreset = presetId;
    
    if (presetId === 'none') {
      resetAdjustments();
    } else {
      const presetDef = presets.find(p => p.id === presetId);
      if (presetDef && presetDef.values) {
        adjustments = { ...adjustments, ...presetDef.values };
      }
    }
    
    await saveAdjustmentsToBackend();
    toast.success($t('edit.title') + ': ' + $t(presets.find(p => p.id === presetId)?.label || ''));
  }

  // Debounced save
  let saveTimeout: any;
  function handleSliderChange() {
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(saveAdjustmentsToBackend, 500);
    
    // Optimistically update the store item so PreviewView reacts immediately
    if (item) {
      item.adjustments = JSON.stringify(adjustments);
      // Update in the store lists
      const updateList = (list: MediaItem[]) => {
        const idx = list.findIndex(i => i.id === item!.id);
        if (idx !== -1) {
          list[idx] = { ...item! };
        }
      };
      
      const currentMedia = get(mediaItems);
      updateList(currentMedia);
      mediaItems.set([...currentMedia]);
    }
  }

  async function saveAdjustmentsToBackend() {
    if (!item) return;
    try {
      await invoke('save_adjustments', { 
        mediaId: item.id, 
        adjustmentsJson: JSON.stringify(adjustments) 
      });
      await invoke('save_applied_preset', {
        mediaId: item.id,
        presetName: activePreset
      });
    } catch (err) {
      console.error('Failed to save adjustments:', err);
    }
  }

  function handleReset() {
    resetAdjustments();
    activePreset = 'none';
    handleSliderChange();
  }

</script>

<aside class="edit-panel">
  <div class="panel-header">
    <h3>{$t('edit.title')}</h3>
    <button class="btn-text" onclick={handleReset}>{$t('edit.reset')}</button>
  </div>

  {#if !item}
    <div class="empty-state">
      <p>Select a photo to edit</p>
    </div>
  {:else}
    <div class="panel-content">
      
      <!-- Presets Section -->
      <section class="edit-section">
        <h4 class="section-title">{$t('edit.presets')}</h4>
        <div class="presets-grid">
          {#each presets as preset}
            <button 
              class="preset-btn" 
              class:active={activePreset === preset.id || (preset.id === 'none' && !activePreset)}
              onclick={() => applyPreset(preset.id)}
            >
              {$t(preset.label)}
            </button>
          {/each}
        </div>
      </section>

      <!-- Basic Section -->
      <section class="edit-section">
        <h4 class="section-title">{$t('edit.basic')}</h4>
        
        <div class="slider-group">
          <div class="slider-header">
            <label for="exposure">{$t('edit.exposure')}</label>
            <span class="value">{adjustments.exposure}</span>
          </div>
          <input type="range" id="exposure" min="-100" max="100" bind:value={adjustments.exposure} oninput={handleSliderChange} />
        </div>

        <div class="slider-group">
          <div class="slider-header">
            <label for="contrast">{$t('edit.contrast')}</label>
            <span class="value">{adjustments.contrast}</span>
          </div>
          <input type="range" id="contrast" min="-100" max="100" bind:value={adjustments.contrast} oninput={handleSliderChange} />
        </div>

        <div class="slider-group">
          <div class="slider-header">
            <label for="highlights">{$t('edit.highlights')}</label>
            <span class="value">{adjustments.highlights}</span>
          </div>
          <input type="range" id="highlights" min="-100" max="100" bind:value={adjustments.highlights} oninput={handleSliderChange} />
        </div>

        <div class="slider-group">
          <div class="slider-header">
            <label for="shadows">{$t('edit.shadows')}</label>
            <span class="value">{adjustments.shadows}</span>
          </div>
          <input type="range" id="shadows" min="-100" max="100" bind:value={adjustments.shadows} oninput={handleSliderChange} />
        </div>
      </section>

      <!-- Color Section -->
      <section class="edit-section">
        <h4 class="section-title">{$t('edit.color')}</h4>

        <div class="slider-group">
          <div class="slider-header">
            <label for="temp">{$t('edit.temperature')}</label>
            <span class="value">{adjustments.temperature}</span>
          </div>
          <input type="range" id="temp" min="-100" max="100" bind:value={adjustments.temperature} oninput={handleSliderChange} class="slider-temp" />
        </div>

        <div class="slider-group">
          <div class="slider-header">
            <label for="tint">{$t('edit.tint')}</label>
            <span class="value">{adjustments.tint}</span>
          </div>
          <input type="range" id="tint" min="-100" max="100" bind:value={adjustments.tint} oninput={handleSliderChange} class="slider-tint" />
        </div>

        <div class="slider-group">
          <div class="slider-header">
            <label for="saturation">{$t('edit.saturation')}</label>
            <span class="value">{adjustments.saturation}</span>
          </div>
          <input type="range" id="saturation" min="-100" max="100" bind:value={adjustments.saturation} oninput={handleSliderChange} />
        </div>

      </section>
    </div>
  {/if}
</aside>

<style>
  .edit-panel {
    width: 100%;
    height: 100%;
    background: var(--bg-secondary);
    border-left: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-header {
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--border-subtle);
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--bg-tertiary);
  }

  .panel-header h3 {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-secondary);
  }

  .btn-text {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-size: var(--text-xs);
    font-weight: 600;
  }

  .btn-text:hover {
    color: var(--accent-light);
  }

  .empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary);
    font-size: var(--text-sm);
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .section-title {
    margin: 0 0 var(--space-3) 0;
    font-size: var(--text-xs);
    font-weight: 700;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .slider-group {
    margin-bottom: var(--space-4);
  }

  .slider-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: var(--space-2);
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .value {
    font-variant-numeric: tabular-nums;
    color: var(--text-primary);
    min-width: 32px;
    text-align: right;
  }

  input[type="range"] {
    width: 100%;
    -webkit-appearance: none;
    background: transparent;
  }

  input[type="range"]::-webkit-slider-runnable-track {
    width: 100%;
    height: 4px;
    background: var(--bg-tertiary);
    border-radius: 2px;
  }

  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    height: 16px;
    width: 16px;
    border-radius: 50%;
    background: white;
    margin-top: -6px;
    cursor: pointer;
    box-shadow: 0 1px 3px rgba(0,0,0,0.5);
  }

  .slider-temp::-webkit-slider-runnable-track {
    background: linear-gradient(to right, #3b82f6, #e5e7eb, #ef4444);
  }

  .slider-tint::-webkit-slider-runnable-track {
    background: linear-gradient(to right, #10b981, #e5e7eb, #d946ef);
  }

  .presets-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-2);
  }

  .preset-btn {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    padding: var(--space-2);
    border-radius: var(--radius-md);
    font-size: var(--text-xs);
    cursor: pointer;
    transition: all 0.2s;
  }

  .preset-btn:hover {
    border-color: var(--text-tertiary);
    color: var(--text-primary);
  }

  .preset-btn.active {
    background: var(--accent-soft);
    border-color: var(--accent);
    color: white;
  }
</style>
