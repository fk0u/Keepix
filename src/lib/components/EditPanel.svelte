<script lang="ts">
  import { t, locale } from '$lib/i18n';
  import { invoke } from '@tauri-apps/api/core';
  import type { MediaItem } from '$lib/types';
  import { currentItem, displayItems, mediaItems } from '$lib/stores/media';
  import { toast } from '$lib/stores/toast';
  import { get } from 'svelte/store';
  import { onMount, untrack } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { editPanelCollapsed } from '$lib/stores/ui';
  import {
    type Adjustments,
    DEFAULT_ADJUSTMENTS,
    PRESETS,
    isDefault,
    mergePreset,
    parseLightroomPreset,
  } from '$lib/services/image-editor';

  let { item }: { item: MediaItem | null } = $props();

  // Edit State
  let adjustments = $state<Adjustments>({ ...DEFAULT_ADJUSTMENTS });
  let activePreset = $state<string | null>(null);
  let activeSection = $state<string>('presets');
  let showBeforeAfter = $state(false);
  let healingMode = $state(false);
  let healBrushSize = $state(20);
  let clipboardAdjustments = $state<Adjustments | null>(null);

  // Load adjustments when item changes
  let prevItemId = '';
  $effect(() => {
    const itemProp = item;
    const itemId = item?.id;
    const itemAdj = item?.adjustments;
    const itemPreset = item?.applied_preset;

    untrack(() => {
      if (itemProp) {
        const currentAdjStr = JSON.stringify(adjustments);
        const isDifferentItem = itemProp.id !== prevItemId;
        const isExternalChange = itemProp.adjustments !== currentAdjStr;

        if (isDifferentItem || isExternalChange) {
          prevItemId = itemProp.id;
          if (itemProp.adjustments) {
            try {
              const parsed = JSON.parse(itemProp.adjustments);
              adjustments = { ...DEFAULT_ADJUSTMENTS, ...parsed };
            } catch {
              resetAdjustments();
            }
          } else {
            resetAdjustments();
          }
          activePreset = itemProp.applied_preset || null;
          if (isDifferentItem) {
            healingMode = false;
          }
        }
      } else {
        resetAdjustments();
        activePreset = null;
        prevItemId = '';
      }
    });
  });

  function resetAdjustments() {
    adjustments = { ...DEFAULT_ADJUSTMENTS };
  }

  async function applyPreset(presetId: string) {
    if (!item) return;
    
    // Commit the preset immediately (stop hovering)
    isHoveringPreset = false;
    originalAdjustmentsBeforeHover = null;
    originalPresetBeforeHover = null;

    activePreset = presetId;

    if (presetId === 'none') {
      resetAdjustments();
    } else {
      const presetDef = PRESETS.find(p => p.id === presetId);
      if (presetDef && presetDef.values) {
        adjustments = mergePreset({ ...DEFAULT_ADJUSTMENTS }, presetDef.values);
      }
    }

    // Optimistically update the store item so PreviewView reacts immediately
    item.adjustments = JSON.stringify(adjustments);
    item.applied_preset = activePreset;
    const updateList = (list: MediaItem[]) => {
      const idx = list.findIndex(i => i.id === item!.id);
      if (idx !== -1) {
        list[idx] = { ...item! };
      }
    };

    const currentMedia = get(mediaItems);
    updateList(currentMedia);
    mediaItems.set([...currentMedia]);

    await saveAdjustmentsToBackend();
    toast.success($t('edit.title') + ': ' + $t(PRESETS.find(p => p.id === presetId)?.labelKey || ''));
  }

  async function importPreset() {
    if (!item) return;
    try {
      const selected = await open({
        multiple: false,
        title: $t('edit.import_preset'),
        filters: [
          {
            name: 'Lightroom Preset / RAW Photo',
            extensions: ['xmp', 'lrtemplate', 'dng', 'arw', 'cr2', 'cr3', 'nef', 'jpg', 'jpeg', 'tif', 'tiff']
          }
        ]
      });

      if (!selected || typeof selected !== 'string') return;

      const presetText = await invoke<string>('extract_xmp_preset', { filePath: selected });

      const parsedValues = parseLightroomPreset(presetText);
      if (Object.keys(parsedValues).length === 0) {
        toast.error($t('edit.import_preset.fail', { err: 'No valid adjustments found in file' }));
        return;
      }

      adjustments = mergePreset({ ...DEFAULT_ADJUSTMENTS }, parsedValues);
      activePreset = null; // Custom preset applied

      // Offer saving to Custom Presets
      const saveToCustom = confirm(
        $locale === 'id' 
          ? 'Preset berhasil diimpor! Simpan ke daftar preset kustom Anda?' 
          : 'Preset imported successfully! Save it to your custom presets list?'
      );
      if (saveToCustom) {
        const fileName = selected.split(/[\\/]/).pop() || '';
        const defaultName = fileName.replace(/\.[^/.]+$/, "").replace(/_/g, " ");
        const presetName = prompt(
          $locale === 'id' ? 'Masukkan nama preset:' : 'Enter preset name:', 
          defaultName
        );
        if (presetName) {
          const id = 'custom_' + Date.now();
          customPresets = [...customPresets, { id, name: presetName, values: parsedValues }];
          saveCustomPresets();
          activePreset = id;
        }
      }

      // Optimistically update store
      item.adjustments = JSON.stringify(adjustments);
      item.applied_preset = activePreset;
      const updateList = (list: MediaItem[]) => {
        const idx = list.findIndex(i => i.id === item!.id);
        if (idx !== -1) {
          list[idx] = { ...item! };
        }
      };
      const currentMedia = get(mediaItems);
      updateList(currentMedia);
      mediaItems.set([...currentMedia]);

      await saveAdjustmentsToBackend();
      toast.success($t('edit.import_preset.success'));
    } catch (err: any) {
      console.error('Failed to import preset:', err);
      toast.error($t('edit.import_preset.fail', { err: err.toString() }));
    }
  }

  // --- Preset Hover Previews ---
  let originalAdjustmentsBeforeHover = $state<Adjustments | null>(null);
  let originalPresetBeforeHover = $state<string | null>(null);
  let isHoveringPreset = $state(false);

  function previewPreset(presetId: string) {
    if (!item) return;

    if (!isHoveringPreset) {
      originalAdjustmentsBeforeHover = { ...adjustments };
      originalPresetBeforeHover = activePreset;
      isHoveringPreset = true;
    }

    if (presetId === 'none') {
      adjustments = { ...DEFAULT_ADJUSTMENTS };
    } else {
      const presetDef = PRESETS.find(p => p.id === presetId);
      if (presetDef && presetDef.values) {
        adjustments = mergePreset({ ...DEFAULT_ADJUSTMENTS }, presetDef.values);
      }
    }

    updateStoreAdjustmentsOnly();
  }

  function previewCustomPreset(preset: CustomPreset) {
    if (!item) return;

    if (!isHoveringPreset) {
      originalAdjustmentsBeforeHover = { ...adjustments };
      originalPresetBeforeHover = activePreset;
      isHoveringPreset = true;
    }

    adjustments = mergePreset({ ...DEFAULT_ADJUSTMENTS }, preset.values);
    updateStoreAdjustmentsOnly();
  }

  function restorePresetPreview() {
    if (!item || !isHoveringPreset || !originalAdjustmentsBeforeHover) return;

    adjustments = { ...originalAdjustmentsBeforeHover };
    activePreset = originalPresetBeforeHover;
    isHoveringPreset = false;
    originalAdjustmentsBeforeHover = null;
    originalPresetBeforeHover = null;

    updateStoreAdjustmentsOnly();
  }

  function updateStoreAdjustmentsOnly() {
    if (!item) return;
    item.adjustments = JSON.stringify(adjustments);
    item.applied_preset = activePreset;
    
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

  // --- Custom Preset CRUD ---
  interface CustomPreset {
    id: string;
    name: string;
    values: Partial<Adjustments>;
  }

  let customPresets = $state<CustomPreset[]>([]);

  function loadCustomPresets() {
    if (typeof localStorage !== 'undefined') {
      const saved = localStorage.getItem('keepix_custom_presets');
      if (saved) {
        try {
          customPresets = JSON.parse(saved);
        } catch {
          customPresets = [];
        }
      }
    }
  }

  function saveCustomPresets() {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('keepix_custom_presets', JSON.stringify(customPresets));
    }
  }

  onMount(() => {
    loadCustomPresets();
  });

  async function applyCustomPreset(preset: CustomPreset) {
    if (!item) return;
    
    isHoveringPreset = false;
    originalAdjustmentsBeforeHover = null;
    originalPresetBeforeHover = null;

    activePreset = preset.id;
    adjustments = mergePreset({ ...DEFAULT_ADJUSTMENTS }, preset.values);

    item.adjustments = JSON.stringify(adjustments);
    item.applied_preset = activePreset;
    const updateList = (list: MediaItem[]) => {
      const idx = list.findIndex(i => i.id === item!.id);
      if (idx !== -1) {
        list[idx] = { ...item! };
      }
    };
    const currentMedia = get(mediaItems);
    updateList(currentMedia);
    mediaItems.set([...currentMedia]);

    await saveAdjustmentsToBackend();
    toast.success($t('edit.title') + ': ' + preset.name);
  }

  function deleteCustomPreset(presetId: string) {
    customPresets = customPresets.filter(p => p.id !== presetId);
    saveCustomPresets();
    if (activePreset === presetId) {
      activePreset = null;
    }
    toast.info($locale === 'id' ? 'Preset kustom dihapus' : 'Custom preset removed');
  }

  function saveCurrentAsPreset() {
    if (!item) return;
    const name = prompt($locale === 'id' ? 'Masukkan nama preset:' : 'Enter preset name:');
    if (!name) return;

    const id = 'custom_' + Date.now();
    const presetValues: Partial<Adjustments> = { ...adjustments };
    delete presetValues.rotation;
    delete presetValues.flipH;
    delete presetValues.flipV;

    customPresets = [...customPresets, { id, name, values: presetValues }];
    saveCustomPresets();
    toast.success($locale === 'id' ? 'Preset kustom disimpan' : 'Custom preset saved');
  }

  // Debounced save
  let saveTimeout: any;
  function handleSliderChange() {
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(saveAdjustmentsToBackend, 500);

    // Optimistically update the store item so PreviewView reacts immediately
    if (item) {
      item.adjustments = JSON.stringify(adjustments);
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
        adjustmentsJson: JSON.stringify(adjustments),
      });
      await invoke('save_applied_preset', {
        mediaId: item.id,
        presetName: activePreset,
      });
    } catch (err) {
      console.error('Failed to save adjustments:', err);
    }
  }

  function handleReset() {
    resetAdjustments();
    activePreset = 'none';
    healingMode = false;
    handleSliderChange();
  }

  function handleRotate(deg: number) {
    adjustments.rotation = ((adjustments.rotation || 0) + deg + 360) % 360;
    handleSliderChange();
  }

  function handleFlipH() {
    adjustments.flipH = !adjustments.flipH;
    handleSliderChange();
  }

  function handleFlipV() {
    adjustments.flipV = !adjustments.flipV;
    handleSliderChange();
  }

  function toggleBeforeAfter() {
    showBeforeAfter = !showBeforeAfter;
    window.dispatchEvent(new CustomEvent('keepix-edit', {
      detail: { type: 'before-after', active: showBeforeAfter },
    }));
  }

  function toggleHealing() {
    healingMode = !healingMode;
    window.dispatchEvent(new CustomEvent('keepix-edit', {
      detail: { type: 'healing-mode', active: healingMode, brushSize: healBrushSize },
    }));
    if (healingMode) {
      toast.info($t('edit.heal.hint'));
    }
  }

  function updateHealBrushSize() {
    if (healingMode) {
      window.dispatchEvent(new CustomEvent('keepix-edit', {
        detail: { type: 'healing-brush-size', brushSize: healBrushSize },
      }));
    }
  }

  function copyAdjustments() {
    clipboardAdjustments = { ...adjustments };
    toast.success($t('edit.copy_success'));
  }

  function pasteAdjustments() {
    if (!clipboardAdjustments || !item) return;
    adjustments = { ...clipboardAdjustments };
    activePreset = null;
    handleSliderChange();
    toast.success($t('edit.paste_success'));
  }

  // Sections for accordion
  const sections = [
    { id: 'presets', labelKey: 'edit.presets', icon: '✨' },
    { id: 'basic', labelKey: 'edit.basic', icon: '☀️' },
    { id: 'color', labelKey: 'edit.color', icon: '🎨' },
    { id: 'detail', labelKey: 'edit.detail', icon: '🔍' },
    { id: 'effects', labelKey: 'edit.effects', icon: '🎬' },
    { id: 'transform', labelKey: 'edit.transform', icon: '🔄' },
    { id: 'tools', labelKey: 'edit.tools', icon: '🔧' },
  ];

  function toggleSection(id: string) {
    activeSection = activeSection === id ? '' : id;
  }

  // Determine if adjustments have changed from defaults
  let hasEdits = $derived(!isDefault(adjustments));

  let activeEditsList = $derived.by(() => {
    const list: { name: string; value: string }[] = [];
    const targetAdjustments = isHoveringPreset && originalAdjustmentsBeforeHover 
      ? originalAdjustmentsBeforeHover 
      : adjustments;

    for (const key of Object.keys(DEFAULT_ADJUSTMENTS) as Array<keyof Adjustments>) {
      if (targetAdjustments[key] !== DEFAULT_ADJUSTMENTS[key]) {
        let name = '';
        let value = '';
        if (key === 'flipH') {
          name = $locale === 'id' ? 'Cermin H' : 'Flip H';
          value = targetAdjustments[key] ? 'ON' : 'OFF';
        } else if (key === 'flipV') {
          name = $locale === 'id' ? 'Cermin V' : 'Flip V';
          value = targetAdjustments[key] ? 'ON' : 'OFF';
        } else {
          name = $t(`edit.${key}`);
          if (name === `edit.${key}`) name = key;
          const val = targetAdjustments[key];
          if (typeof val === 'number') {
            value = val > 0 && key !== 'rotation' ? `+${val}` : val.toString();
            if (key === 'rotation') value = `${val}°`;
          } else {
            value = String(val);
          }
        }
        list.push({ name, value });
      }
    }
    return list;
  });

  onMount(() => {
    function handleAction(e: Event) {
      const detail = (e as CustomEvent).detail;
      if (detail === 'reset-adjustments') {
        handleReset();
      } else if (detail === 'copy-adjustments') {
        copyAdjustments();
      } else if (detail === 'paste-adjustments') {
        pasteAdjustments();
      }
    }
    window.addEventListener('keepix-action', handleAction);
    return () => window.removeEventListener('keepix-action', handleAction);
  });
</script>

<aside class="edit-panel">
  <div class="panel-header">
    <h3>{$t('edit.title')}</h3>
    <div class="header-actions">
      {#if hasEdits}
        <span class="edit-indicator" title="Edits applied">●</span>
      {/if}
      <button class="btn-icon" onclick={copyAdjustments} title={$t('edit.copy')}>
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="4" y="4" width="9" height="9" rx="1"/>
          <path d="M10 4V2.5A1.5 1.5 0 008.5 1H2.5A1.5 1.5 0 001 2.5v6A1.5 1.5 0 002.5 10H4"/>
        </svg>
      </button>
      <button class="btn-icon" onclick={pasteAdjustments} title={$t('edit.paste')} disabled={!clipboardAdjustments}>
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="3" y="1" width="8" height="12" rx="1"/>
          <path d="M5 1V0h4v1"/>
          <line x1="5" y1="6" x2="9" y2="6"/>
          <line x1="5" y1="8" x2="9" y2="8"/>
          <line x1="5" y1="10" x2="7" y2="10"/>
        </svg>
      </button>
      <button class="btn-text" onclick={handleReset}>{$t('edit.reset')}</button>
      <button class="btn-icon collapse-btn" onclick={() => editPanelCollapsed.set(true)} title={$t('edit.minimize')}>
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="5 2 10 7 5 12"/>
        </svg>
      </button>
    </div>
  </div>

  {#if !item}
    <div class="empty-state">
      <p>{$t('edit.empty')}</p>
    </div>
  {:else if item.file_type !== 'photo'}
    <div class="empty-state">
      <p>{$t('edit.video_note')}</p>
    </div>
  {:else}
    <div class="panel-content">
      {#if activeEditsList.length > 0}
        <div class="active-adjustments-hud">
          <div class="hud-header">
            <span>{$locale === 'id' ? 'Penyesuaian Aktif' : 'Active Adjustments'}</span>
            <button class="hud-reset-btn" onclick={handleReset}>{$locale === 'id' ? 'Reset Semua' : 'Reset All'}</button>
          </div>
          <div class="hud-chips">
            {#each activeEditsList as edit}
              <span class="hud-chip">
                <span class="hud-chip-name">{edit.name}:</span>
                <span class="hud-chip-value">{edit.value}</span>
              </span>
            {/each}
          </div>
        </div>
      {/if}

      {#each sections as sec}
        <section class="edit-section">
          <button class="section-toggle" class:open={activeSection === sec.id} onclick={() => toggleSection(sec.id)}>
            <span class="section-icon">{sec.icon}</span>
            <span class="section-label">{$t(sec.labelKey)}</span>
            <svg class="chevron" width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 4 6 7 9 4"/>
            </svg>
          </button>

          {#if activeSection === sec.id}
            <div class="section-body">

              {#if sec.id === 'presets'}
                <div class="presets-grid">
                  {#each PRESETS as preset}
                    <button
                      class="preset-btn"
                      class:active={activePreset === preset.id || (preset.id === 'none' && !activePreset)}
                      onclick={() => applyPreset(preset.id)}
                      onmouseenter={() => previewPreset(preset.id)}
                      onmouseleave={restorePresetPreview}
                    >
                      <span class="preset-label">{$t(preset.labelKey)}</span>
                    </button>
                  {/each}
                </div>

                {#if customPresets.length > 0}
                  <div class="custom-presets-title">{$locale === 'id' ? 'Preset Kustom' : 'Custom Presets'}</div>
                  <div class="presets-grid">
                    {#each customPresets as preset}
                      <div class="custom-preset-wrapper">
                        <button
                          class="preset-btn custom-preset-btn"
                          class:active={activePreset === preset.id}
                          onclick={() => applyCustomPreset(preset)}
                          onmouseenter={() => previewCustomPreset(preset)}
                          onmouseleave={restorePresetPreview}
                        >
                          <span class="preset-label">{preset.name}</span>
                        </button>
                        <button class="delete-preset-btn" onclick={(e) => { e.stopPropagation(); deleteCustomPreset(preset.id); }} title="Delete Preset">
                          <svg width="10" height="10" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2.5">
                            <line x1="4" y1="4" x2="12" y2="12"/>
                            <line x1="12" y1="4" x2="4" y2="12"/>
                          </svg>
                        </button>
                      </div>
                    {/each}
                  </div>
                {/if}

                <div class="preset-actions-row">
                  <button class="action-preset-btn" onclick={saveCurrentAsPreset}>
                    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M13 3H3v10h10V5l-2-2z"/>
                      <circle cx="8" cy="8" r="3"/>
                    </svg>
                    <span>{$locale === 'id' ? 'Simpan Baru...' : 'Save Current...'}</span>
                  </button>
                  <button class="action-preset-btn" onclick={importPreset}>
                    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M8 12V2M4 6l4-4 4 4M2 14h12"/>
                    </svg>
                    <span>{$locale === 'id' ? 'Impor...' : 'Import...'}</span>
                  </button>
                </div>

              {:else if sec.id === 'basic'}
                <div class="slider-group">
                  <div class="slider-header">
                    <label for="ed-exposure">{$t('edit.exposure')}</label>
                    <span class="value">{adjustments.exposure}</span>
                  </div>
                  <input type="range" id="ed-exposure" min="-100" max="100" bind:value={adjustments.exposure} oninput={handleSliderChange} />
                </div>
                <div class="slider-group">
                  <div class="slider-header">
                    <label for="ed-contrast">{$t('edit.contrast')}</label>
                    <span class="value">{adjustments.contrast}</span>
                  </div>
                  <input type="range" id="ed-contrast" min="-100" max="100" bind:value={adjustments.contrast} oninput={handleSliderChange} />
                </div>
                <div class="slider-group">
                  <div class="slider-header">
                    <label for="ed-highlights">{$t('edit.highlights')}</label>
                    <span class="value">{adjustments.highlights}</span>
                  </div>
                  <input type="range" id="ed-highlights" min="-100" max="100" bind:value={adjustments.highlights} oninput={handleSliderChange} />
                </div>
                <div class="slider-group">
                  <div class="slider-header">
                    <label for="ed-shadows">{$t('edit.shadows')}</label>
                    <span class="value">{adjustments.shadows}</span>
                  </div>
                  <input type="range" id="ed-shadows" min="-100" max="100" bind:value={adjustments.shadows} oninput={handleSliderChange} />
                </div>

              {:else if sec.id === 'color'}
                <div class="slider-group">
                  <div class="slider-header">
                    <label for="ed-temp">{$t('edit.temperature')}</label>
                    <span class="value">{adjustments.temperature}</span>
                  </div>
                  <input type="range" id="ed-temp" min="-100" max="100" bind:value={adjustments.temperature} oninput={handleSliderChange} class="slider-temp" />
                </div>
                <div class="slider-group">
                  <div class="slider-header">
                    <label for="ed-tint">{$t('edit.tint')}</label>
                    <span class="value">{adjustments.tint}</span>
                  </div>
                  <input type="range" id="ed-tint" min="-100" max="100" bind:value={adjustments.tint} oninput={handleSliderChange} class="slider-tint" />
                </div>
                <div class="slider-group">
                  <div class="slider-header">
                    <label for="ed-saturation">{$t('edit.saturation')}</label>
                    <span class="value">{adjustments.saturation}</span>
                  </div>
                  <input type="range" id="ed-saturation" min="-100" max="100" bind:value={adjustments.saturation} oninput={handleSliderChange} />
                </div>
                <div class="slider-group">
                  <div class="slider-header">
                    <label for="ed-vibrance">{$t('edit.vibrance')}</label>
                    <span class="value">{adjustments.vibrance}</span>
                  </div>
                  <input type="range" id="ed-vibrance" min="-100" max="100" bind:value={adjustments.vibrance} oninput={handleSliderChange} />
                </div>

              {:else if sec.id === 'detail'}
                <div class="slider-group">
                  <div class="slider-header">
                    <label for="ed-sharpen">{$t('edit.sharpen')}</label>
                    <span class="value">{adjustments.sharpen}</span>
                  </div>
                  <input type="range" id="ed-sharpen" min="0" max="100" bind:value={adjustments.sharpen} oninput={handleSliderChange} />
                </div>
                <div class="slider-group">
                  <div class="slider-header">
                    <label for="ed-clarity">{$t('edit.clarity')}</label>
                    <span class="value">{adjustments.clarity}</span>
                  </div>
                  <input type="range" id="ed-clarity" min="-100" max="100" bind:value={adjustments.clarity} oninput={handleSliderChange} />
                </div>

              {:else if sec.id === 'effects'}
                <div class="slider-group">
                  <div class="slider-header">
                    <label for="ed-vignette">{$t('edit.vignette')}</label>
                    <span class="value">{adjustments.vignette}</span>
                  </div>
                  <input type="range" id="ed-vignette" min="0" max="100" bind:value={adjustments.vignette} oninput={handleSliderChange} />
                </div>
                <div class="slider-group">
                  <div class="slider-header">
                    <label for="ed-grain">{$t('edit.grain')}</label>
                    <span class="value">{adjustments.grain}</span>
                  </div>
                  <input type="range" id="ed-grain" min="0" max="100" bind:value={adjustments.grain} oninput={handleSliderChange} />
                </div>

              {:else if sec.id === 'transform'}
                <div class="transform-grid">
                  <button class="transform-btn" onclick={() => handleRotate(-90)} title={$t('edit.rotate_ccw')}>
                    <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5">
                      <path d="M4 7l-3-3 3-3"/>
                      <path d="M1 4h10a5 5 0 015 5v0a5 5 0 01-5 5H6"/>
                    </svg>
                    <span>{$t('edit.rotate_ccw')}</span>
                  </button>
                  <button class="transform-btn" onclick={() => handleRotate(90)} title={$t('edit.rotate_cw')}>
                    <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5">
                      <path d="M14 7l3-3-3-3"/>
                      <path d="M17 4H7a5 5 0 00-5 5v0a5 5 0 005 5h5"/>
                    </svg>
                    <span>{$t('edit.rotate_cw')}</span>
                  </button>
                  <button class="transform-btn" class:active={adjustments.flipH} onclick={handleFlipH} title={$t('edit.flip_h')}>
                    <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5">
                      <path d="M9 2v14"/>
                      <path d="M3 6l-2 3 2 3"/>
                      <path d="M15 6l2 3-2 3"/>
                    </svg>
                    <span>{$t('edit.flip_h')}</span>
                  </button>
                  <button class="transform-btn" class:active={adjustments.flipV} onclick={handleFlipV} title={$t('edit.flip_v')}>
                    <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5">
                      <path d="M2 9h14"/>
                      <path d="M6 3L9 1l3 2"/>
                      <path d="M6 15l3 2 3-2"/>
                    </svg>
                    <span>{$t('edit.flip_v')}</span>
                  </button>
                </div>
                {#if adjustments.rotation !== 0}
                  <div class="transform-status">
                    {$t('edit.rotation')}: {adjustments.rotation}°
                  </div>
                {/if}

              {:else if sec.id === 'tools'}
                <div class="tools-section">
                  <button class="tool-btn" class:active={showBeforeAfter} onclick={toggleBeforeAfter}>
                    <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5">
                      <rect x="1" y="2" width="16" height="14" rx="2"/>
                      <line x1="9" y1="2" x2="9" y2="16"/>
                      <text x="4" y="11" font-size="5" fill="currentColor" stroke="none">B</text>
                      <text x="11" y="11" font-size="5" fill="currentColor" stroke="none">A</text>
                    </svg>
                    <span>{$t('edit.before_after')}</span>
                    <kbd>B</kbd>
                  </button>

                  <button class="tool-btn" class:active={healingMode} onclick={toggleHealing}>
                    <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5">
                      <circle cx="9" cy="9" r="7"/>
                      <line x1="9" y1="5" x2="9" y2="13"/>
                      <line x1="5" y1="9" x2="13" y2="9"/>
                    </svg>
                    <span>{$t('edit.heal')}</span>
                  </button>

                  {#if healingMode}
                    <div class="heal-options">
                      <div class="slider-group compact">
                        <div class="slider-header">
                          <label for="heal-size">{$t('edit.heal.brush_size')}</label>
                          <span class="value">{healBrushSize}px</span>
                        </div>
                        <input type="range" id="heal-size" min="5" max="80" bind:value={healBrushSize} oninput={updateHealBrushSize} />
                      </div>
                      <div class="heal-hint">
                        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5">
                          <circle cx="6" cy="6" r="5"/>
                          <line x1="6" y1="4" x2="6" y2="6"/>
                          <circle cx="6" cy="8" r="0.5" fill="currentColor"/>
                        </svg>
                        {$t('edit.heal.hint')}
                      </div>
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          {/if}
        </section>
      {/each}
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
    gap: var(--space-2);
  }

  .panel-header h3 {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-secondary);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .edit-indicator {
    color: var(--accent);
    font-size: 10px;
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .btn-icon {
    background: none;
    border: 1px solid transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 4px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    transition: all 0.15s;
  }

  .btn-icon:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--bg-tertiary);
    border-color: var(--border-subtle);
  }

  .btn-icon:disabled {
    opacity: 0.3;
    cursor: default;
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
    padding: var(--space-4);
    text-align: center;
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  /* Accordion sections */
  .edit-section {
    border-bottom: 1px solid var(--border-subtle);
  }

  .section-toggle {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: var(--text-xs);
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    transition: background 0.15s;
  }

  .section-toggle:hover {
    background: rgba(255, 255, 255, 0.03);
    color: var(--text-primary);
  }

  .section-icon {
    font-size: 12px;
    line-height: 1;
  }

  .section-label {
    flex: 1;
    text-align: left;
  }

  .chevron {
    transition: transform 0.2s;
  }

  .section-toggle.open .chevron {
    transform: rotate(180deg);
  }

  .section-body {
    padding: 0 var(--space-4) var(--space-4);
    animation: slideDown 0.15s ease-out;
  }

  @keyframes slideDown {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  /* Sliders */
  .slider-group {
    margin-bottom: var(--space-3);
  }

  .slider-group.compact {
    margin-bottom: var(--space-2);
  }

  .slider-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: var(--space-1);
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .value {
    font-variant-numeric: tabular-nums;
    color: var(--text-primary);
    min-width: 32px;
    text-align: right;
    font-size: var(--text-xs);
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
    height: 14px;
    width: 14px;
    border-radius: 50%;
    background: white;
    margin-top: -5px;
    cursor: pointer;
    box-shadow: 0 1px 3px rgba(0,0,0,0.5);
    transition: transform 0.1s;
  }

  input[type="range"]::-webkit-slider-thumb:hover {
    transform: scale(1.15);
  }

  .slider-temp::-webkit-slider-runnable-track {
    background: linear-gradient(to right, #3b82f6, #e5e7eb, #ef4444);
  }

  .slider-tint::-webkit-slider-runnable-track {
    background: linear-gradient(to right, #10b981, #e5e7eb, #d946ef);
  }

  /* Presets Grid */
  .presets-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-2);
  }

  .preset-btn {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    padding: var(--space-2) var(--space-2);
    border-radius: var(--radius-md);
    font-size: 10px;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    min-height: 36px;
    justify-content: center;
  }

  .preset-btn:hover {
    border-color: var(--text-tertiary);
    color: var(--text-primary);
    transform: translateY(-1px);
  }

  .preset-btn.active {
    background: var(--accent-soft);
    border-color: var(--accent);
    color: white;
  }

  .preset-label {
    text-align: center;
    line-height: 1.2;
  }

  /* Transform Grid */
  .transform-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-2);
  }

  .transform-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: var(--space-3) var(--space-2);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 10px;
    transition: all 0.15s;
  }

  .transform-btn:hover {
    border-color: var(--text-tertiary);
    color: var(--text-primary);
  }

  .transform-btn.active {
    background: var(--accent-soft);
    border-color: var(--accent);
    color: white;
  }

  .transform-status {
    margin-top: var(--space-2);
    text-align: center;
    font-size: var(--text-xs);
    color: var(--accent);
    font-weight: 600;
  }

  /* Tools Section */
  .tools-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .tool-btn {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-3);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: var(--text-sm);
    transition: all 0.15s;
    width: 100%;
    text-align: left;
  }

  .tool-btn:hover {
    border-color: var(--text-tertiary);
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.05);
  }

  .tool-btn.active {
    background: var(--accent-soft);
    border-color: var(--accent);
    color: white;
  }

  .tool-btn kbd {
    margin-left: auto;
    padding: 1px 6px;
    font-size: 9px;
    font-family: var(--font-mono);
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 3px;
    color: var(--text-tertiary);
  }

  .heal-options {
    padding: var(--space-2) 0 0;
  }

  .heal-hint {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 10px;
    color: var(--text-tertiary);
    padding: var(--space-2);
    background: rgba(59, 130, 246, 0.08);
    border-radius: var(--radius-sm);
    border: 1px solid rgba(59, 130, 246, 0.15);
    margin-top: var(--space-2);
  }

  .custom-presets-title {
    margin-top: var(--space-4);
    margin-bottom: var(--space-2);
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-tertiary);
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: 4px;
    grid-column: span 2;
  }

  .custom-preset-wrapper {
    position: relative;
    display: flex;
    align-items: stretch;
  }

  .custom-preset-wrapper:hover .delete-preset-btn {
    opacity: 1;
  }

  .custom-preset-btn {
    width: 100%;
  }

  .delete-preset-btn {
    position: absolute;
    top: 4px;
    right: 4px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    color: var(--text-tertiary);
    cursor: pointer;
    border-radius: 50%;
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    opacity: 0;
    transition: all 0.15s ease-in-out;
  }

  .delete-preset-btn:hover {
    background: #ef4444;
    border-color: #ef4444;
    color: white;
  }

  .preset-actions-row {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-3);
    grid-column: span 2;
    width: 100%;
  }

  .action-preset-btn {
    flex: 1;
    background: var(--bg-tertiary);
    border: 1px dashed var(--border-strong);
    color: var(--text-secondary);
    padding: var(--space-2);
    border-radius: var(--radius-md);
    font-size: var(--text-xs);
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
  }

  .action-preset-btn:hover {
    border-style: solid;
    border-color: var(--accent);
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.02);
  }

  .active-adjustments-hud {
    background: rgba(255, 255, 255, 0.02);
    border-bottom: 1px solid var(--border-subtle);
    padding: var(--space-3) var(--space-4);
  }

  .hud-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-2);
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
  }

  .hud-reset-btn {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-size: 10px;
    font-weight: 600;
    padding: 0;
  }

  .hud-reset-btn:hover {
    color: var(--accent-light);
    text-decoration: underline;
  }

  .hud-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .hud-chip {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    border-radius: 4px;
    padding: 2px 6px;
    font-size: 10px;
    font-family: var(--font-mono);
    display: flex;
    gap: 4px;
  }

  .hud-chip-name {
    color: var(--text-tertiary);
  }

  .hud-chip-value {
    color: var(--accent);
    font-weight: 600;
  }
</style>
