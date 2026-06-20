<script lang="ts">
  import { onMount } from 'svelte';

  let {
    minSizes = [200, 300, 250],
    defaultSizes = [250, 600, 300],
    left,
    center,
    right,
    rightCollapsed = false
  }: {
    minSizes?: number[];
    defaultSizes?: number[];
    left?: import('svelte').Snippet;
    center?: import('svelte').Snippet;
    right?: import('svelte').Snippet;
    rightCollapsed?: boolean;
  } = $props();
  
  let container = $state<HTMLDivElement>();
  let sizes = $state<number[]>([]);

  $effect.pre(() => {
    if (sizes.length === 0) {
      sizes = [...defaultSizes];
    }
  });
  
  let isDragging = $state(false);
  let dragIndex = $state(-1);
  let startX = $state(0);
  let startSizes = $state<number[]>([]);

  function onPointerDown(e: PointerEvent, index: number) {
    e.preventDefault();
    isDragging = true;
    dragIndex = index;
    startX = e.clientX;
    startSizes = [...sizes];
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';

    window.addEventListener('pointermove', onPointerMove);
    window.addEventListener('pointerup', onPointerUp);
  }

  function onPointerMove(e: PointerEvent) {
    if (!isDragging || dragIndex === -1) return;

    const deltaX = e.clientX - startX;
    const newSizes = [...startSizes];

    const leftPanelIdx = dragIndex;
    const rightPanelIdx = dragIndex + 1;

    // Calculate new sizes
    let newLeftSize = startSizes[leftPanelIdx] + deltaX;
    let newRightSize = startSizes[rightPanelIdx] - deltaX;

    // Enforce minimum sizes
    const minLeft = minSizes[leftPanelIdx] || 100;
    const minRight = minSizes[rightPanelIdx] || 100;

    if (newLeftSize < minLeft) {
      newLeftSize = minLeft;
      newRightSize = startSizes[leftPanelIdx] + startSizes[rightPanelIdx] - minLeft;
    } else if (newRightSize < minRight) {
      newRightSize = minRight;
      newLeftSize = startSizes[leftPanelIdx] + startSizes[rightPanelIdx] - minRight;
    }

    newSizes[leftPanelIdx] = newLeftSize;
    newSizes[rightPanelIdx] = newRightSize;

    sizes = newSizes;
  }

  function onPointerUp() {
    isDragging = false;
    dragIndex = -1;
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
    window.removeEventListener('pointermove', onPointerMove);
    window.removeEventListener('pointerup', onPointerUp);
  }

  onMount(() => {
    // Dynamically calculate sizes if container is available
    if (container) {
      const totalWidth = container.clientWidth;
      const initialSizes = [...defaultSizes];
      // Distribute the middle panel to take remaining space
      const fixedWidth = initialSizes[0] + initialSizes[2];
      if (totalWidth > fixedWidth) {
        initialSizes[1] = totalWidth - fixedWidth;
        sizes = initialSizes;
      }
    }
  });

</script>

<div class="split-pane-container" bind:this={container}>
  <!-- Panel 0 (Sidebar) -->
  <div class="split-panel" style="width: {sizes[0]}px;">
    {@render left?.()}
  </div>

  <!-- Resizer 0 -->
  <div 
    class="split-resizer" 
    onpointerdown={(e) => onPointerDown(e, 0)} 
    role="separator" 
    aria-label="Resize panels"
    aria-valuenow={sizes[0]}
    tabindex="0"
  >
    <div class="resizer-handle"></div>
  </div>

  <!-- Panel 1 (Main/Center) -->
  <div class="split-panel" style="width: {rightCollapsed ? 'auto' : (sizes[1] + 'px')}; flex: 1;">
    {@render center?.()}
  </div>

  <!-- Resizer 1 -->
  {#if !rightCollapsed}
    <div 
      class="split-resizer" 
      onpointerdown={(e) => onPointerDown(e, 1)} 
      role="separator" 
      aria-label="Resize panels"
      aria-valuenow={sizes[0] + sizes[1]}
      tabindex="0"
    >
      <div class="resizer-handle"></div>
    </div>
  {/if}

  <!-- Panel 2 (Edit Panel) -->
  {#if !rightCollapsed}
    <div class="split-panel" style="width: {sizes[2]}px;">
      {@render right?.()}
    </div>
  {/if}
</div>

<style>
  .split-pane-container {
    display: flex;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .split-panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    height: 100%;
  }

  .split-resizer {
    width: 4px;
    background: transparent;
    cursor: col-resize;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
    transition: background 0.15s;
    flex-shrink: 0;
    border-left: 1px solid var(--border-subtle);
    border-right: 1px solid var(--border-subtle);
  }

  .split-resizer:hover, .split-resizer:active {
    background: var(--accent);
  }

  .resizer-handle {
    width: 0;
    height: 100%;
  }
</style>
