<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { t, locale, setLanguage, type Language } from '$lib/i18n';
  import { invoke } from '@tauri-apps/api/core';

  let { show = false, onClose }: { show?: boolean; onClose: () => void } = $props();

  let currentLang = $state<Language>($locale);
  let cacheLimit = $state(300);

  // Load existing settings when modal opens
  $effect(() => {
    if (show) {
      currentLang = $locale;
      invoke('get_setting', { key: 'cache_limit' }).then((val) => {
        if (val) cacheLimit = parseInt(val as string, 10) || 300;
      });
    }
  });

  async function handleSave() {
    await setLanguage(currentLang);
    await invoke('set_setting', { key: 'cache_limit', value: cacheLimit.toString() });
    onClose();
  }
</script>

{#if show}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="modal-backdrop" transition:fade={{ duration: 150 }} onclick={onClose} role="dialog">
    <div class="modal-container" transition:scale={{ start: 0.95, duration: 200 }} onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2>{$t('settings.title')}</h2>
        <button class="close-btn" onclick={onClose}>
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <div class="modal-content">
        <!-- Language Setting -->
        <div class="setting-group">
          <h3>{$t('settings.language')}</h3>
          <div class="setting-row">
            <select bind:value={currentLang} class="setting-select">
              <option value="en">{$t('settings.language.en')}</option>
              <option value="id">{$t('settings.language.id')}</option>
            </select>
          </div>
        </div>

        <!-- Performance Setting -->
        <div class="setting-group">
          <h3>{$t('settings.performance')}</h3>
          <div class="setting-row">
            <label for="cache-limit">{$t('settings.cache_limit')}</label>
            <div class="slider-container">
              <input type="range" id="cache-limit" min="100" max="1000" step="50" bind:value={cacheLimit} />
              <span class="slider-value">{cacheLimit} MB</span>
            </div>
            <p class="setting-help">{$t('settings.cache.info')}</p>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-ghost" onclick={onClose}>{$t('settings.cancel')}</button>
        <button class="btn btn-primary" onclick={handleSave}>{$t('settings.save')}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
  }

  .modal-container {
    background: #1e1e1e;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    width: 480px;
    max-width: 90vw;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.4);
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.5);
    cursor: pointer;
    border-radius: 4px;
    padding: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .close-btn:hover {
    color: white;
    background: rgba(255, 255, 255, 0.1);
  }

  .modal-content {
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .setting-group h3 {
    margin: 0 0 12px 0;
    font-size: 0.9rem;
    color: rgba(255, 255, 255, 0.5);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .setting-row {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .setting-row label {
    font-size: 0.95rem;
  }

  .setting-select {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: white;
    padding: 10px 12px;
    border-radius: 6px;
    font-size: 0.95rem;
    outline: none;
    cursor: pointer;
  }

  .setting-select:focus {
    border-color: #6366f1;
  }

  .slider-container {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .slider-container input[type="range"] {
    flex: 1;
    accent-color: #6366f1;
  }

  .slider-value {
    font-variant-numeric: tabular-nums;
    min-width: 60px;
    text-align: right;
    color: rgba(255, 255, 255, 0.7);
  }

  .setting-help {
    margin: 0;
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.4);
  }

  .modal-footer {
    padding: 16px 24px;
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    background: rgba(0, 0, 0, 0.2);
    border-bottom-left-radius: 12px;
    border-bottom-right-radius: 12px;
  }

  .btn {
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    border: none;
  }

  .btn-ghost {
    background: transparent;
    color: rgba(255, 255, 255, 0.7);
  }

  .btn-ghost:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }

  .btn-primary {
    background: #6366f1;
    color: white;
  }

  .btn-primary:hover {
    background: #4f46e5;
  }
</style>
