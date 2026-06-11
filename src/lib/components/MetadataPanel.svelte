<script lang="ts">
  import type { ExifData, MediaItem } from '$lib/types';
  import { formatFileSize } from '$lib/types';

  let {
    exifData,
    item,
    onClose,
  }: {
    exifData: ExifData | null;
    item: MediaItem | null;
    onClose: () => void;
  } = $props();
</script>

<aside class="metadata-panel">
  <div class="panel-header">
    <h3 class="panel-title">Info</h3>
    <button class="btn btn-ghost btn-icon" onclick={onClose}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18"/>
        <line x1="6" y1="6" x2="18" y2="18"/>
      </svg>
    </button>
  </div>

  {#if item}
    <div class="panel-body">
      <!-- File info -->
      <section class="meta-section">
        <h4 class="meta-section-title">File</h4>
        <div class="meta-grid">
          <div class="meta-row">
            <span class="meta-label">Name</span>
            <span class="meta-value truncate">{item.file_name}</span>
          </div>
          <div class="meta-row">
            <span class="meta-label">Size</span>
            <span class="meta-value">{formatFileSize(item.file_size)}</span>
          </div>
          <div class="meta-row">
            <span class="meta-label">Type</span>
            <span class="meta-value">{exifData?.file_format ?? item.file_type}</span>
          </div>
          {#if exifData?.width && exifData?.height}
            <div class="meta-row">
              <span class="meta-label">Dimensions</span>
              <span class="meta-value">{exifData.width} × {exifData.height}</span>
            </div>
          {/if}
        </div>
      </section>

      {#if exifData}
        <!-- Camera info -->
        {#if exifData.camera_model || exifData.lens_model}
          <section class="meta-section">
            <h4 class="meta-section-title">Camera</h4>
            <div class="meta-grid">
              {#if exifData.camera_make}
                <div class="meta-row">
                  <span class="meta-label">Make</span>
                  <span class="meta-value">{exifData.camera_make}</span>
                </div>
              {/if}
              {#if exifData.camera_model}
                <div class="meta-row">
                  <span class="meta-label">Model</span>
                  <span class="meta-value">{exifData.camera_model}</span>
                </div>
              {/if}
              {#if exifData.lens_model}
                <div class="meta-row">
                  <span class="meta-label">Lens</span>
                  <span class="meta-value truncate">{exifData.lens_model}</span>
                </div>
              {/if}
            </div>
          </section>
        {/if}

        <!-- Exposure -->
        {#if exifData.aperture || exifData.shutter_speed || exifData.iso}
          <section class="meta-section">
            <h4 class="meta-section-title">Exposure</h4>

            <!-- Exposure summary strip -->
            <div class="exposure-strip">
              {#if exifData.focal_length}
                <span class="exposure-tag">{exifData.focal_length}</span>
              {/if}
              {#if exifData.aperture}
                <span class="exposure-tag">{exifData.aperture}</span>
              {/if}
              {#if exifData.shutter_speed}
                <span class="exposure-tag">{exifData.shutter_speed}</span>
              {/if}
              {#if exifData.iso}
                <span class="exposure-tag">ISO {exifData.iso}</span>
              {/if}
            </div>

            <div class="meta-grid">
              {#if exifData.metering_mode}
                <div class="meta-row">
                  <span class="meta-label">Metering</span>
                  <span class="meta-value">{exifData.metering_mode}</span>
                </div>
              {/if}
              {#if exifData.white_balance}
                <div class="meta-row">
                  <span class="meta-label">WB</span>
                  <span class="meta-value">{exifData.white_balance}</span>
                </div>
              {/if}
              {#if exifData.flash}
                <div class="meta-row">
                  <span class="meta-label">Flash</span>
                  <span class="meta-value">{exifData.flash}</span>
                </div>
              {/if}
            </div>
          </section>
        {/if}

        <!-- Date -->
        {#if exifData.date_taken}
          <section class="meta-section">
            <h4 class="meta-section-title">Date</h4>
            <div class="meta-grid">
              <div class="meta-row">
                <span class="meta-label">Taken</span>
                <span class="meta-value">{exifData.date_taken}</span>
              </div>
            </div>
          </section>
        {/if}

        <!-- GPS -->
        {#if exifData.gps_latitude !== null && exifData.gps_longitude !== null}
          <section class="meta-section">
            <h4 class="meta-section-title">Location</h4>
            <div class="meta-grid">
              <div class="meta-row">
                <span class="meta-label">Lat</span>
                <span class="meta-value">{exifData.gps_latitude?.toFixed(6)}</span>
              </div>
              <div class="meta-row">
                <span class="meta-label">Lng</span>
                <span class="meta-value">{exifData.gps_longitude?.toFixed(6)}</span>
              </div>
              {#if exifData.gps_altitude}
                <div class="meta-row">
                  <span class="meta-label">Alt</span>
                  <span class="meta-value">{exifData.gps_altitude.toFixed(1)}m</span>
                </div>
              {/if}
            </div>
          </section>
        {/if}
      {/if}
    </div>
  {:else}
    <div class="panel-empty">
      <p>Select an item to view info</p>
    </div>
  {/if}
</aside>

<style>
  .metadata-panel {
    width: 260px;
    height: 100%;
    background: var(--bg-secondary);
    border-left: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    animation: slideUp 0.2s ease forwards;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-3);
    border-bottom: 1px solid var(--border-subtle);
  }

  .panel-title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-primary);
  }

  .panel-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-2);
  }

  .meta-section {
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--border-subtle);
  }

  .meta-section:last-child {
    border-bottom: none;
  }

  .meta-section-title {
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: var(--space-2);
    padding: 0 var(--space-1);
  }

  .meta-grid {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .meta-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 2px var(--space-1);
    border-radius: var(--radius-sm);
  }

  .meta-row:hover {
    background: var(--surface-card);
  }

  .meta-label {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    flex-shrink: 0;
    min-width: 70px;
  }

  .meta-value {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    text-align: right;
    font-family: var(--font-mono);
    max-width: 150px;
  }

  .exposure-strip {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    margin-bottom: var(--space-2);
    padding: 0 var(--space-1);
  }

  .exposure-tag {
    display: inline-flex;
    padding: 2px 6px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }

  .panel-empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary);
    font-size: var(--text-sm);
  }
</style>
