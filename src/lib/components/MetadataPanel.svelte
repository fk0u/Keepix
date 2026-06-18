<script lang="ts">
  import type { ExifData, MediaItem } from '$lib/types';
  import { formatFileSize } from '$lib/types';
  import { t } from '$lib/i18n';

  let {
    exifData,
    item,
    onClose,
  }: {
    exifData: ExifData | null;
    item: MediaItem | null;
    onClose: () => void;
  } = $props();

  function getBrandLogoText(make: string): string {
    const m = make.toLowerCase();
    if (m.includes('sony')) return 'SONY';
    if (m.includes('canon')) return 'Canon';
    if (m.includes('nikon')) return 'Nikon';
    if (m.includes('fujifilm') || m.includes('fuji')) return 'FUJIFILM';
    if (m.includes('leica')) return 'Leica';
    if (m.includes('panasonic')) return 'LUMIX';
    if (m.includes('olympus')) return 'OLYMPUS';
    if (m.includes('apple')) return ' Apple';
    return make.toUpperCase();
  }

  // Calculate Exposure Value (EV) at ISO 100 (EV100) and current ISO
  function getCalculatedEV(aperture: string | null, shutter: string | null, iso: string | null): string | null {
    if (!aperture || !shutter) return null;
    try {
      // Parse Aperture (e.g., "f/2.8" or "2.8")
      const fNum = parseFloat(aperture.replace(/[^\d.]/g, ''));
      if (isNaN(fNum) || fNum <= 0) return null;

      // Parse Shutter Speed (e.g., "1/125" or "0.008" or "1/125 sec")
      let tSpeed = 1.0;
      if (shutter.includes('/')) {
        const parts = shutter.split('/');
        const num = parseFloat(parts[0]);
        const den = parseFloat(parts[1].replace(/[^\d.]/g, ''));
        tSpeed = num / den;
      } else {
        tSpeed = parseFloat(shutter.replace(/[^\d.]/g, ''));
      }
      if (isNaN(tSpeed) || tSpeed <= 0) return null;

      // EV = log2(N^2 / t)
      const ev100 = Math.log2((fNum * fNum) / tSpeed);

      // If ISO is available, compute EV at current ISO
      let currentIso = 100;
      if (iso) {
        currentIso = parseFloat(iso.replace(/[^\d.]/g, ''));
      }
      if (isNaN(currentIso) || currentIso <= 0) currentIso = 100;

      const evCurrent = ev100 + Math.log2(currentIso / 100);

      return `EV ${ev100.toFixed(1)} @ ISO 100 / EV ${evCurrent.toFixed(1)}`;
    } catch {
      return null;
    }
  }
</script>

{#snippet renderBrandLogo(make: string)}
  {@const m = make.toLowerCase()}
  <div class="brand-logo-container">
    {#if m.includes('sony')}
      <svg width="80" height="16" viewBox="0 0 100 20" class="brand-svg">
        <text x="50%" y="15" text-anchor="middle" font-family="'Times New Roman', serif" font-weight="bold" font-size="16" fill="var(--text-primary)" letter-spacing="4">SONY</text>
      </svg>
    {:else if m.includes('canon')}
      <svg width="80" height="18" viewBox="0 0 100 24" class="brand-svg">
        <text x="50%" y="18" text-anchor="middle" font-family="'Impact', sans-serif" font-weight="900" font-style="italic" font-size="20" fill="#ef4444" letter-spacing="-1.5">Canon</text>
      </svg>
    {:else if m.includes('nikon')}
      <svg width="80" height="18" viewBox="0 0 100 24" class="brand-svg">
        <polygon points="5,2 95,2 85,22 5,22" fill="#eab308" />
        <text x="45%" y="17" text-anchor="middle" font-family="Impact, sans-serif" font-weight="900" font-style="italic" font-size="15" fill="black">Nikon</text>
      </svg>
    {:else if m.includes('fujifilm') || m.includes('fuji')}
      <svg width="90" height="16" viewBox="0 0 100 20" class="brand-svg">
        <text x="50%" y="14" text-anchor="middle" font-family="'Arial Black', sans-serif" font-weight="900" font-size="11" fill="var(--text-primary)" letter-spacing="1">FUJIFILM</text>
        <rect x="42" y="1" width="16" height="2" fill="#ef4444" />
      </svg>
    {:else if m.includes('leica')}
      <svg width="70" height="22" viewBox="0 0 100 28" class="brand-svg">
        <circle cx="50" cy="14" r="13" fill="#ef4444" />
        <text x="50%" y="18" text-anchor="middle" font-family="'Brush Script MT', cursive, sans-serif" font-style="italic" font-weight="bold" font-size="13" fill="white">Leica</text>
      </svg>
    {:else if m.includes('panasonic') || m.includes('lumix')}
      <svg width="80" height="16" viewBox="0 0 100 20" class="brand-svg">
        <text x="50%" y="15" text-anchor="middle" font-family="var(--font-sans)" font-weight="900" font-size="15" fill="var(--text-primary)" letter-spacing="1">LUMIX</text>
      </svg>
    {:else if m.includes('olympus')}
      <svg width="90" height="16" viewBox="0 0 110 20" class="brand-svg">
        <text x="55" y="14" text-anchor="middle" font-family="Georgia, serif" font-weight="bold" font-size="13" fill="var(--text-primary)" letter-spacing="1.5">OLYMPUS</text>
        <line x1="15" y1="17" x2="95" y2="17" stroke="#eab308" stroke-width="1.5" />
      </svg>
    {:else if m.includes('apple')}
      <span class="brand-text-fallback"> Apple</span>
    {:else}
      <span class="brand-text-fallback">{getBrandLogoText(make)}</span>
    {/if}
  </div>
{/snippet}

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
            <span class="meta-value truncate" title={item.file_name}>{item.file_name}</span>
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
        <!-- Equipment Card Section -->
        {#if exifData.camera_model || exifData.lens_model}
          <section class="meta-section camera-section">
            <h4 class="meta-section-title">Equipment</h4>
            
            {#if exifData.camera_make}
              {@render renderBrandLogo(exifData.camera_make)}
            {/if}

            <div class="equipment-visuals">
              {#if exifData.camera_model}
                <div class="equip-detail-card">
                  <div class="equip-art">
                    <!-- Vector SVG representation of camera body -->
                    <svg width="48" height="48" viewBox="0 0 100 100" class="equip-vector-svg">
                      <path d="M15,32 h15 l5,-7 h30 l5,7 h15 a5,5 0 0,1 5,5 v43 a5,5 0 0,1 -5,5 h-80 a5,5 0 0,1 -5,-5 v-43 a5,5 0 0,1 5,-5 z" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linejoin="round" />
                      <circle cx="50" cy="55" r="21" fill="none" stroke="currentColor" stroke-width="2.5" />
                      <rect x="42" y="49" width="16" height="12" rx="1" fill="none" stroke="currentColor" stroke-width="1.5" />
                      <circle cx="37" cy="41" r="2" fill="#ef4444" />
                      <rect x="22" y="27" width="8" height="5" fill="currentColor" opacity="0.6" />
                      <circle cx="78" cy="26" r="3" fill="currentColor" opacity="0.6" />
                      <path d="M38,25 h24 l4,7 h-32 z" fill="currentColor" opacity="0.1" />
                    </svg>
                  </div>
                  <div class="equip-details">
                    <span class="equip-label">Body</span>
                    <span class="equip-name truncate" title={exifData.camera_model}>{exifData.camera_model}</span>
                  </div>
                </div>
              {/if}
              
              {#if exifData.lens_model}
                <div class="equip-detail-card">
                  <div class="equip-art">
                    <!-- Vector SVG representation of lens barrel -->
                    <svg width="48" height="48" viewBox="0 0 100 100" class="equip-vector-svg">
                      <path d="M32,15 h36 v6 h-36 z M27,21 h46 v24 h-46 z M30,45 h40 v14 h-40 z M22,59 h56 v18 h-56 z M35,77 h30 v8 h-30 z" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linejoin="round" />
                      <line x1="32" y1="28" x2="68" y2="28" stroke="currentColor" stroke-width="1.5" stroke-dasharray="2,2" />
                      <line x1="32" y1="36" x2="68" y2="36" stroke="currentColor" stroke-width="1.5" stroke-dasharray="2,2" />
                      <line x1="34" y1="52" x2="66" y2="52" stroke="currentColor" stroke-width="1.5" stroke-dasharray="3,3" />
                      <line x1="26" y1="67" x2="74" y2="67" stroke="currentColor" stroke-width="1" />
                      <path d="M38,81 Q50,77 62,81" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.6" />
                      <line x1="42" y1="12" x2="58" y2="12" stroke="currentColor" stroke-width="2" />
                    </svg>
                  </div>
                  <div class="equip-details">
                    <span class="equip-label">Lens</span>
                    <span class="equip-name truncate" title={exifData.lens_model}>{exifData.lens_model}</span>
                  </div>
                </div>
              {/if}
            </div>
          </section>
        {/if}

        <!-- Exposure Parameters -->
        {#if exifData.aperture || exifData.shutter_speed || exifData.iso}
          <section class="meta-section">
            <h4 class="meta-section-title">Exposure</h4>

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
              {#if exifData.exposure_compensation}
                <div class="meta-row">
                  <span class="meta-label">Exp Comp</span>
                  <span class="meta-value">{exifData.exposure_compensation}</span>
                </div>
              {/if}
              {#if getCalculatedEV(exifData.aperture, exifData.shutter_speed, exifData.iso)}
                <div class="meta-row">
                  <span class="meta-label">Calculated EV</span>
                  <span class="meta-value ev-val">{getCalculatedEV(exifData.aperture, exifData.shutter_speed, exifData.iso)}</span>
                </div>
              {/if}
              {#if exifData.metering_mode}
                <div class="meta-row">
                  <span class="meta-label">Metering</span>
                  <span class="meta-value">{exifData.metering_mode}</span>
                </div>
              {/if}
              {#if exifData.white_balance}
                <div class="meta-row">
                  <span class="meta-label">White Balance</span>
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

        <!-- Date & Software details -->
        {#if exifData.date_taken || exifData.software}
          <section class="meta-section">
            <h4 class="meta-section-title">Creation</h4>
            <div class="meta-grid">
              {#if exifData.date_taken}
                <div class="meta-row">
                  <span class="meta-label">Date Taken</span>
                  <span class="meta-value">{exifData.date_taken}</span>
                </div>
              {/if}
              {#if exifData.software}
                <div class="meta-row">
                  <span class="meta-label">Software</span>
                  <span class="meta-value truncate" title={exifData.software}>{exifData.software}</span>
                </div>
              {/if}
            </div>
          </section>
        {/if}

        <!-- GPS location details -->
        {#if exifData.gps_latitude !== null && exifData.gps_longitude !== null}
          <section class="meta-section">
            <h4 class="meta-section-title">Location Coordinates</h4>
            <div class="meta-grid">
              <div class="meta-row">
                <span class="meta-label">Latitude</span>
                <span class="meta-value">{exifData.gps_latitude?.toFixed(6)}°</span>
              </div>
              <div class="meta-row">
                <span class="meta-label">Longitude</span>
                <span class="meta-value">{exifData.gps_longitude?.toFixed(6)}°</span>
              </div>
              {#if exifData.gps_altitude}
                <div class="meta-row">
                  <span class="meta-label">Altitude</span>
                  <span class="meta-value">{exifData.gps_altitude.toFixed(1)}m</span>
                </div>
              {/if}
              <div class="map-link-container">
                <a 
                  class="btn btn-ghost btn-sm map-btn"
                  href="https://www.google.com/maps/search/?api=1&query={exifData.gps_latitude},{exifData.gps_longitude}"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <span class="map-icon">📍</span> Open in Google Maps
                </a>
              </div>
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
    width: 280px;
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
    padding: var(--space-3);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-primary);
  }

  .panel-title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .panel-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .meta-section {
    padding-bottom: var(--space-3);
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
    letter-spacing: 0.8px;
    margin-bottom: var(--space-2);
  }

  .meta-grid {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .meta-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 3px var(--space-1);
    border-radius: var(--radius-sm);
  }

  .meta-row:hover {
    background: var(--surface-card);
  }

  .meta-label {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .meta-value {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    text-align: right;
    font-family: var(--font-mono);
    max-width: 170px;
    word-break: break-all;
  }

  .ev-val {
    color: var(--accent-light);
    font-weight: 600;
  }

  .exposure-strip {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: var(--space-2);
  }

  .exposure-tag {
    display: inline-flex;
    padding: 2px 6px;
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--text-primary);
    font-weight: 500;
  }

  .panel-empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary);
    font-size: var(--text-sm);
  }

  /* Equipment Section Improvements */
  .camera-section {
    background: rgba(255, 255, 255, 0.01);
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-2);
    border: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .brand-logo-container {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 24px;
    margin-bottom: var(--space-2);
    border-bottom: 1px solid rgba(255, 255, 255, 0.03);
    padding-bottom: var(--space-2);
  }

  .brand-svg {
    opacity: 0.95;
    filter: drop-shadow(0 0 2px rgba(255, 255, 255, 0.1));
  }

  .brand-text-fallback {
    font-family: var(--font-sans);
    font-weight: 800;
    font-size: var(--text-md);
    letter-spacing: 1.5px;
    color: var(--text-primary);
  }

  .equipment-visuals {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .equip-detail-card {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    background: var(--bg-primary);
    padding: var(--space-2);
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .equip-art {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    background: var(--bg-secondary);
    border-radius: var(--radius-md);
    color: var(--text-secondary);
    border: 1px solid rgba(255, 255, 255, 0.03);
  }

  .equip-vector-svg {
    color: var(--text-secondary);
    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
  }

  .equip-details {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    flex: 1;
  }

  .equip-label {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-tertiary);
    font-weight: 700;
  }

  .equip-name {
    font-size: 11px;
    color: var(--text-primary);
    font-weight: 500;
  }

  /* Map Link Button */
  .map-link-container {
    margin-top: var(--space-2);
    display: flex;
    justify-content: center;
  }

  .map-btn {
    width: 100%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    border: 1px solid var(--border-subtle);
    font-size: 11px;
    padding: 6px;
    color: var(--text-secondary);
    background: var(--bg-primary);
  }

  .map-btn:hover {
    color: white;
    background: var(--bg-hover);
    border-color: var(--border-default);
  }

  .map-icon {
    font-size: 12px;
  }
</style>
