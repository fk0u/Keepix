<script lang="ts">
  import { scanProgress, isScanning } from '$lib/stores/project';

  $effect(() => {
    // Just subscribe to reactivity
    $scanProgress;
  });
</script>

{#if $isScanning && $scanProgress}
  <div class="progress-overlay">
    <div class="progress-content glass-card">
      <div class="progress-header">
        <span class="progress-phase">
          {#if $scanProgress.phase === 'scanning'}
            📂 Scanning files...
          {:else if $scanProgress.phase === 'importing'}
            💾 Importing to database...
          {:else if $scanProgress.phase === 'thumbnails'}
            🖼️ Generating thumbnails...
          {:else}
            ✅ Complete!
          {/if}
        </span>
        {#if $scanProgress.total > 0}
          <span class="progress-numbers">
            {$scanProgress.current} / {$scanProgress.total}
          </span>
        {/if}
      </div>

      <div class="progress-bar">
        <div
          class="progress-bar-fill"
          style="width: {$scanProgress.total > 0
            ? ($scanProgress.current / $scanProgress.total * 100)
            : 0}%"
        ></div>
      </div>

      <div class="progress-file truncate">
        {$scanProgress.current_file}
      </div>
    </div>
  </div>
{/if}

<style>
  .progress-overlay {
    position: absolute;
    top: var(--space-4);
    left: 50%;
    transform: translateX(-50%);
    z-index: 50;
    animation: slideDown var(--transition-base) forwards;
  }

  .progress-content {
    padding: var(--space-4) var(--space-6);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-width: 360px;
    max-width: 500px;
  }

  .progress-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
  }

  .progress-phase {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-primary);
  }

  .progress-numbers {
    font-size: var(--text-xs);
    font-family: var(--font-mono);
    color: var(--text-tertiary);
  }

  .progress-file {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    font-family: var(--font-mono);
    max-width: 100%;
  }
</style>
