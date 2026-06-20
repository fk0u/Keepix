<script lang="ts">
  import type { MediaItem } from '$lib/types';
  import { toAssetUrl } from '$lib/services/tauri-bridge';
  import { currentItem } from '$lib/stores/media';

  let {
    item,
  }: {
    item: MediaItem;
  } = $props();

  let videoElement = $state<HTMLVideoElement>();
  let videoPaused = $state(true);
  let videoCurrentTime = $state(0);
  let videoDuration = $state(0);
  let videoVolume = $state(0.7);
  let videoMuted = $state(false);
  let videoLoop = $state(false);
  let videoPlaybackRate = $state(1.0);
  let videoError = $state<string | null>(null);

  // Reset play state when item changes
  $effect(() => {
    if (item.id) {
      videoPaused = true;
      videoCurrentTime = 0;
      videoPlaybackRate = 1.0;
      videoError = null;
    }
  });

  function handleWindowKeydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    
    // Ensure this player is the active item selected in preview view
    if ($currentItem?.id !== item.id) return;

    if (e.key === ' ' || e.key === 'k' || e.key === 'K') {
      e.preventDefault();
      e.stopPropagation();
      videoPaused = !videoPaused;
    } else if (e.key === '[' || e.key === 'j' || e.key === 'J') {
      e.preventDefault();
      e.stopPropagation();
      if (videoElement) {
        videoPaused = true;
        // Step back 1 frame (approx. 1/30 seconds)
        videoElement.currentTime = Math.max(0, videoElement.currentTime - 1/30);
      }
    } else if (e.key === ']' || e.key === 'l' || e.key === 'L') {
      e.preventDefault();
      e.stopPropagation();
      if (videoElement) {
        videoPaused = true;
        // Step forward 1 frame (approx. 1/30 seconds)
        videoElement.currentTime = Math.min(videoElement.duration || 9999, videoElement.currentTime + 1/30);
      }
    } else if (e.key === 'm' || e.key === 'M') {
      e.preventDefault();
      e.stopPropagation();
      videoMuted = !videoMuted;
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

  function handleVideoError() {
    videoError = 'Failed to load video';
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="video-player-container" onclick={(e) => { e.stopPropagation(); videoPaused = !videoPaused; }} role="presentation">
  {#if videoError || item.width === -1}
    <div class="video-error-overlay glass-card" onclick={(e) => e.stopPropagation()} role="presentation">
      <div class="error-icon">⚠️</div>
      <div class="error-title">Pratinjau Video Terbatas</div>
      <p class="error-desc">
        Format kontainer (.mkv / .mov) atau codec video ini (seperti H.265 / HEVC) mungkin tidak didukung secara bawaan oleh sistem Windows WebView2 Anda.
      </p>
      <p class="error-help">
        <strong>Solusi:</strong> Silakan pasang <strong>"HEVC Video Extensions"</strong> dari Microsoft Store untuk mengaktifkan pemutaran video secara langsung di Keepix.
      </p>
      <div class="error-actions">
        <span class="unsupported-badge">Selesai Memindai & Culling Tetap Aktif</span>
      </div>
    </div>
  {:else}
    <!-- svelte-ignore a11y-media-has-caption -->
    <video
      bind:this={videoElement}
      src={toAssetUrl(item.file_path)}
      class="preview-video-element"
      bind:paused={videoPaused}
      bind:currentTime={videoCurrentTime}
      bind:duration={videoDuration}
      bind:volume={videoVolume}
      bind:muted={videoMuted}
      bind:playbackRate={videoPlaybackRate}
      loop={videoLoop}
      onerror={handleVideoError}
    ></video>
  {/if}

  <!-- Metadata Badge Overlay -->
  <div class="video-meta-badge glass-card" onclick={(e) => e.stopPropagation()} role="presentation">
    <span class="meta-format">{item.file_path.split('.').pop()?.toUpperCase()}</span>
    {#if item.width && item.width > 0}
      <span class="meta-res">{item.width} × {item.height}</span>
    {/if}
  </div>

  <!-- Controls overlay -->
  <div class="video-controls glass-card" onclick={(e) => e.stopPropagation()} role="presentation">
    <!-- Frame Step Back -->
    <button 
      class="btn btn-ghost btn-icon step-btn" 
      onclick={(e) => { e.stopPropagation(); if (videoElement) { videoPaused = true; videoElement.currentTime = Math.max(0, videoElement.currentTime - 1/30); }}} 
      title="Mundur 1 Bingkai (J / [)"
      aria-label="Step back frame"
    >
      <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
        <polygon points="19,6 19,18 10,12"/>
        <line x1="6" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2.5"/>
      </svg>
    </button>

    <!-- Play/Pause -->
    <button class="btn btn-ghost btn-icon play-btn" onclick={handlePlayClick} aria-label={videoPaused ? "Play" : "Pause"}>
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

    <!-- Frame Step Forward -->
    <button 
      class="btn btn-ghost btn-icon step-btn" 
      onclick={(e) => { e.stopPropagation(); if (videoElement) { videoPaused = true; videoElement.currentTime = Math.min(videoElement.duration || 9999, videoElement.currentTime + 1/30); }}} 
      title="Maju 1 Bingkai (L / ])"
      aria-label="Step forward frame"
    >
      <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
        <polygon points="5,6 5,18 14,12"/>
        <line x1="18" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2.5"/>
      </svg>
    </button>

    <!-- Progress bar -->
    <div class="video-progress-container">
      <input
        type="range"
        min="0"
        max={videoDuration || 100}
        step="0.05"
        bind:value={videoCurrentTime}
        class="video-progress-slider"
        aria-label="Video Seek Position"
      />
    </div>

    <!-- Time display -->
    <span class="video-time">
      {formatTime(videoCurrentTime)} / {formatTime(videoDuration)}
    </span>

    <!-- Playback Speed Dropdown -->
    <div class="video-speed-container">
      <select bind:value={videoPlaybackRate} class="video-speed-select" aria-label="Playback speed select">
        <option value={0.5}>0.5x</option>
        <option value={1.0}>1.0x</option>
        <option value={1.5}>1.5x</option>
        <option value={2.0}>2.0x</option>
      </select>
    </div>

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

  .video-meta-badge {
    position: absolute;
    top: var(--space-4);
    left: var(--space-4);
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) var(--space-3);
    background: rgba(20, 20, 23, 0.7);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    backdrop-filter: blur(10px);
    z-index: 10;
  }

  .meta-format {
    color: var(--accent-light);
  }

  .meta-res {
    opacity: 0.6;
    border-left: 1px solid rgba(255, 255, 255, 0.2);
    padding-left: var(--space-2);
    font-family: var(--font-mono);
  }

  .video-controls {
    position: absolute;
    bottom: var(--space-4);
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    min-width: 480px;
    max-width: 95%;
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

  .video-speed-container {
    display: flex;
    align-items: center;
  }

  .video-speed-select {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: var(--text-secondary);
    font-size: 10px;
    padding: 2px 4px;
    border-radius: var(--radius-sm);
    outline: none;
    cursor: pointer;
  }

  .video-speed-select:hover {
    color: white;
    border-color: rgba(255, 255, 255, 0.3);
  }

  .video-speed-select option {
    background: #1c1c1f;
    color: white;
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

  .play-btn, .mute-btn, .loop-btn, .step-btn {
    min-width: 28px;
    min-height: 28px;
    padding: 4px;
  }

  /* Codec/Format Error Overlay */
  .video-error-overlay {
    position: absolute;
    inset: var(--space-4);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: rgba(18, 18, 20, 0.85);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
    text-align: center;
    z-index: 12;
    backdrop-filter: blur(15px);
  }

  .error-icon {
    font-size: var(--text-2xl);
    margin-bottom: var(--space-3);
    animation: pulse 2s infinite;
  }

  .error-title {
    font-size: var(--text-md);
    font-weight: 700;
    color: white;
    margin-bottom: var(--space-2);
  }

  .error-desc {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    max-width: 420px;
    margin-bottom: var(--space-3);
    line-height: 1.4;
  }

  .error-help {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    background: rgba(255, 255, 255, 0.03);
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid rgba(255, 255, 255, 0.05);
    max-width: 440px;
    margin-bottom: var(--space-4);
    line-height: 1.4;
  }

  .unsupported-badge {
    font-size: 10px;
    font-weight: 700;
    color: var(--color-simpan);
    background: rgba(34, 197, 94, 0.15);
    border: 1px solid rgba(34, 197, 94, 0.3);
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-full);
    letter-spacing: 0.5px;
  }

  @keyframes pulse {
    0% { opacity: 0.8; transform: scale(1); }
    50% { opacity: 1; transform: scale(1.05); }
    100% { opacity: 0.8; transform: scale(1); }
  }
</style>
