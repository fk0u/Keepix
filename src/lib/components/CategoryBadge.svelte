<script lang="ts">
  import { getCategoryColor } from '$lib/types';
  import { t } from '$lib/i18n';

  let {
    categoryId,
    compact = false,
  }: {
    categoryId: number;
    compact?: boolean;
  } = $props();

  const name = $derived.by(() => {
    if (categoryId === 1) return $t('ref.trash');
    if (categoryId === 2) return $t('ref.best');
    if (categoryId === 3) return $t('ref.draft');
    if (categoryId === 4) return $t('ref.review');
    return $t('sidebar.uncategorized');
  });

  const color = $derived(getCategoryColor(categoryId));

  const badgeClass = $derived((() => {
    switch (categoryId) {
      case 1: return 'badge-buang';
      case 2: return 'badge-simpan';
      case 3: return 'badge-draft';
      case 4: return 'badge-review';
      default: return '';
    }
  })());
</script>

<span class="badge {badgeClass}" class:compact>
  {#if compact}
    <span class="dot" style="background: {color}"></span>
  {:else}
    {name}
  {/if}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .badge.compact {
    padding: 3px;
    border-radius: var(--radius-full);
    min-width: auto;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full);
  }
</style>
