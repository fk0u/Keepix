<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { t } from '$lib/i18n';

  export let show = false;
  export let onClose: () => void;
</script>

{#if show}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="modal-backdrop" transition:fade={{ duration: 150 }} onclick={onClose} role="dialog">
    <div class="modal-container" transition:scale={{ start: 0.95, duration: 200 }} onclick={(e) => e.stopPropagation()}>
      <div class="about-hero">
        <div class="logo">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
            <polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline>
            <line x1="12" y1="22.08" x2="12" y2="12"></line>
          </svg>
        </div>
        <h2>{$t('app.name')}</h2>
        <span class="version">{$t('app.version')}</span>
      </div>

      <div class="about-content">
        <p>{$t('about.desc')}</p>
        
        <div class="credits">
          <div class="credit-item">
            <span>Powered by</span>
            <strong>Rust & Tauri</strong>
          </div>
          <div class="credit-item">
            <span>UI Framework</span>
            <strong>SvelteKit</strong>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-primary" onclick={onClose}>{$t('about.close')}</button>
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
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
  }

  .modal-container {
    background: #1e1e1e;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 16px;
    width: 400px;
    max-width: 90vw;
    box-shadow: 0 32px 64px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .about-hero {
    background: linear-gradient(135deg, #4f46e5 0%, #ec4899 100%);
    padding: 48px 24px 32px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    color: white;
  }

  .logo {
    width: 64px;
    height: 64px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 16px;
    backdrop-filter: blur(4px);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
  }

  .logo svg {
    width: 32px;
    height: 32px;
  }

  .about-hero h2 {
    margin: 0 0 8px 0;
    font-size: 1.75rem;
    font-weight: 700;
    letter-spacing: -0.5px;
  }

  .version {
    font-family: monospace;
    background: rgba(0, 0, 0, 0.2);
    padding: 4px 12px;
    border-radius: 999px;
    font-size: 0.85rem;
  }

  .about-content {
    padding: 32px 24px;
    text-align: center;
  }

  .about-content p {
    color: rgba(255, 255, 255, 0.8);
    line-height: 1.6;
    margin: 0 0 24px 0;
  }

  .credits {
    display: flex;
    gap: 16px;
    justify-content: center;
  }

  .credit-item {
    display: flex;
    flex-direction: column;
    background: rgba(255, 255, 255, 0.05);
    padding: 12px 16px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.05);
    flex: 1;
  }

  .credit-item span {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.4);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 4px;
  }

  .credit-item strong {
    font-size: 0.95rem;
    color: rgba(255, 255, 255, 0.9);
  }

  .modal-footer {
    padding: 16px 24px;
    display: flex;
    justify-content: stretch;
  }

  .btn {
    flex: 1;
    padding: 12px;
    border-radius: 8px;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    border: none;
  }

  .btn-primary {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }

  .btn-primary:hover {
    background: rgba(255, 255, 255, 0.15);
  }
</style>
