<script lang="ts">
  import '../app.css';
  import { toasts, dismissToast } from '$lib/stores/toast';
  import { initLanguage } from '$lib/i18n';
  import { onMount } from 'svelte';

  onMount(() => {
    initLanguage();
  });

  // Snippet-based slot for Svelte 5
  let { children } = $props<{ children: any }>();
</script>

<div id="app-root">
  {@render children()}
</div>

<!-- Toast notifications -->
{#if $toasts.length > 0}
  <div class="toast-container">
    {#each $toasts as t (t.id)}
      <div class="toast toast-{t.type}" role="alert">
        <span class="toast-icon">
          {#if t.type === 'success'}✓
          {:else if t.type === 'error'}✕
          {:else if t.type === 'warning'}⚠
          {:else}ℹ
          {/if}
        </span>
        <span class="toast-message">{t.message}</span>
        {#if t.undoAction}
          <button class="toast-undo" onclick={() => { t.undoAction?.(); dismissToast(t.id); }}>
            Undo
          </button>
        {/if}
        <button class="toast-close" onclick={() => dismissToast(t.id)}>✕</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  #app-root {
    width: 100%;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .toast-icon {
    font-size: var(--text-md);
    flex-shrink: 0;
  }

  .toast-message {
    flex: 1;
  }

  .toast-undo {
    background: none;
    border: none;
    color: var(--accent);
    cursor: pointer;
    font-size: var(--text-sm);
    font-weight: 600;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    font-family: var(--font-sans);
  }

  .toast-undo:hover {
    background: var(--accent-soft);
  }

  .toast-close {
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: var(--text-sm);
    padding: 2px;
    line-height: 1;
    font-family: var(--font-sans);
  }

  .toast-close:hover {
    color: var(--text-secondary);
  }

  :global(.toast-success) {
    border-left: 3px solid var(--color-simpan) !important;
  }

  :global(.toast-error) {
    border-left: 3px solid var(--color-buang) !important;
  }

  :global(.toast-warning) {
    border-left: 3px solid var(--color-draft) !important;
  }

  :global(.toast-info) {
    border-left: 3px solid var(--accent) !important;
  }
</style>
