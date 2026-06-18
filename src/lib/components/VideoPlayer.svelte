<script lang="ts">
  import type { MediaItem } from '$lib/types';
  import { toAssetUrl } from '$lib/services/tauri-bridge';
  import { currentItem } from '$lib/stores/media';

  let {
    item,
  }: {
    item: MediaItem;
  } = $props();

  let videoPaused = $state(true);
  let videoCurrentTime = $state(0);
  let videoDuration = $state(0);
  let videoVolume = $state(0.7);
  let videoMuted = $state(false);
  let videoLoop = $state(false);

  // Reset play state when item changes
  $effect(() => {
    if (item.id) {
      videoPaused = true;
      videoCurrentTime = 0;
    }
  });

  function handleWindowKeydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    if (e.key === ' ' && $currentItem?.id === item.id) {
      e.preventDefault();
      e.stopPropagation();
      videoPaused = !videoPaused;
    }
  }

  function formatTime(seconds: number): string {
    if (isNaN(seconds) || seconds === Infinity) return '0:00';
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs < 10 ? '0' : ''}${secs}`;
  }

  function handlePlayClick(e: MouseEvent) {
    e.stopPropagation();
    videoPaused = !videoPaused;
  }

  function handleMuteClick(e: MouseEvent) {
    e.stopPropagation();
    videoMuted = !videoMuted;
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="video-player-container" onclick={(e) => { e.stopPropagation(); videoPaused = !videoPaused; }} role="presentation">
  <!-- svelte-ignore a11y-media-has-caption -->
  <video
    src={toAssetUrl(item.file_path)}
    class="preview-video-element"
    bind:paused={videoPaused}
    bind:currentTime={videoCurrentTime}
    bind:duration={videoDuration}
    bind:volume={videoVolume}
    bind:muted={videoMuted}
    loop={videoLoop}
  ></video>

  <!-- Controls overlay -->
  <div class="video-controls glass-card" onclick={(e) => e.stopPropagation()} role="presentation">
    <button class="btn btn-ghost btn-icon play-btn" onclick={handlePlayClick} aria-label={videoPaused ? "Pause" : "Play"}>
      {#if videoPaused}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <polygon points="5,3 19,12 5,21"/>
        </svg>
      {:else}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <rect x="6" y="4" width="4" height="16"/>
          <rect x="14" y="4" width="4" height="16"/>
        </svg>
      {/if}
    </button>

    <!-- Progress bar -->
    <div class="video-progress-container">
      <input
        type="range"
        min="0"
        max={videoDuration || 100}
        step="0.1"
        bind:value={videoCurrentTime}
        class="video-progress-slider"
        aria-label="Video Seek Position"
      />
    </div>

    <!-- Time display -->
    <span class="video-time">
      {formatTime(videoCurrentTime)} / {formatTime(videoDuration)}
    </span>

    <!-- Loop -->
    <button 
      class="btn btn-ghost btn-icon loop-btn" 
      class:active={videoLoop}
      onclick={(e) => { e.stopPropagation(); videoLoop = !videoLoop; }}
      data-tooltip="Loop Video"
      aria-label="Loop video"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M17 2.1l4 4-4 4"/>
        <path d="M3 12a9 9 0 0 1 15-6.7L21 6"/>
        <path d="M7 21.9l-4-4 4-4"/>
        <path d="M21 12a9 9 0 0 1-15 6.7L3 18"/>
      </svg>
    </button>

    <!-- Volume -->
    <div class="video-volume-container">
      <button class="btn btn-ghost btn-icon mute-btn" onclick={handleMuteClick} aria-label={videoMuted ? "Unmute" : "Mute"}>
        {#if videoMuted || videoVolume === 0}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M11 5L6 9H2v6h4l5 4V5z"/>
            <line x1="23" y1="9" x2="17" y2="15"/>
            <line x1="17" y1="9" x2="23" y2="15"/>
          </svg>
        {:else}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M11 5L6 9H2v6h4l5 4V5z"/>
            <path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>
          </svg>
        {/if}
      </button>
      <input
        type="range"
        min="0"
        max="1"
        step="0.05"
        bind:value={videoVolume}
        class="video-volume-slider"
        aria-label="Volume level"
      />
    </div>
  </div>
</div>

<style>
  .video-player-container {
    position: relative;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #000;
    overflow: hidden;
  }

  .preview-video-element {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    outline: none;
  }

  .video-controls {
    position: absolute;
    bottom: var(--space-4);
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-4);
    min-width: 320px;
    max-width: 90%;
    z-index: 10;
    pointer-events: auto;
    background: var(--surface-glass);
    backdrop-filter: blur(20px);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
  }

  .video-progress-container {
    flex: 1;
    display: flex;
    align-items: center;
  }

  .video-progress-slider {
    width: 100%;
    height: 4px;
    appearance: none;
    background: rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-full);
    outline: none;
    cursor: pointer;
  }

  .video-progress-slider::-webkit-slider-thumb {
    appearance: none;
    width: 10px;
    height: 10px;
    border-radius: var(--radius-full);
    background: var(--accent);
    cursor: pointer;
    transition: transform var(--transition-fast);
  }

  .video-progress-slider::-webkit-slider-thumb:hover {
    transform: scale(1.3);
  }

  .video-time {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    min-width: 70px;
    text-align: center;
  }

  .video-volume-container {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  .video-volume-slider {
    width: 60px;
    height: 3px;
    appearance: none;
    background: rgba(255, 255, 255, 0.2);
    border-radius: var(--radius-full);
    outline: none;
    cursor: pointer;
  }

  .video-volume-slider::-webkit-slider-thumb {
    appearance: none;
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full);
    background: var(--text-secondary);
    cursor: pointer;
  }

  .loop-btn.active {
    color: var(--accent);
    background: var(--accent-soft);
  }

  .play-btn, .mute-btn, .loop-btn {
    min-width: 28px;
    min-height: 28px;
    padding: 4px;
  }
</style>
