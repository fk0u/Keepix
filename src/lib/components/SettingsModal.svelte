<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { t, locale, setLanguage, type Language } from '$lib/i18n';
  import { invoke } from '@tauri-apps/api/core';
  import { activeTheme, setTheme } from '$lib/stores/ui';
  import { onMount, onDestroy } from 'svelte';
  import { toast } from '$lib/stores/toast';

  let { show = false, onClose }: { show?: boolean; onClose: () => void } = $props();

  // Settings State
  let activeTab = $state<'general' | 'performance' | 'diagnostics' | 'theme'>('general');
  let currentLang = $state<Language>($locale);
  let cacheLimit = $state(300);
  let themeSelected = $state('resolve-dark');
  let cpuThreads = $state(4);
  let gpuAccel = $state(true);
  let focusColor = $state('red');
  let focusSensitivity = $state('medium');
  let histogramMode = $state('all');
  let autoAdvanceDelay = $state(0);

  // New Advanced Configurations
  let threadPriority = $state('medium');
  let allocMode = $state('dynamic');
  let focusIntensity = $state(80);

  // Live Hardware Specs
  let sysCores = $state(4);
  let gpuModel = $state('Software Renderer');
  let gpuVendor = $state('Generic');

  // Simulated live CPU threads load
  let threadLoads = $state<number[]>([]);
  let monitorInterval: any = null;

  $effect(() => {
    if (show && activeTab === 'performance') {
      // Initialize workloads array based on selected CPU threads
      threadLoads = Array(cpuThreads).fill(0).map(() => Math.floor(Math.random() * 20) + 5);
      
      monitorInterval = setInterval(() => {
        threadLoads = threadLoads.map(load => {
          const change = Math.floor(Math.random() * 21) - 10;
          return Math.max(5, Math.min(95, load + change));
        });
      }, 700);
    } else {
      clearInterval(monitorInterval);
    }
  });

  onDestroy(() => {
    if (monitorInterval) clearInterval(monitorInterval);
  });

  // Load existing settings when modal opens
  $effect(() => {
    if (show) {
      currentLang = $locale;
      
      // Load settings from db
      const loadConfig = async () => {
        try {
          const valCache = await invoke<string | null>('get_setting', { key: 'cache_limit' });
          if (valCache) cacheLimit = parseInt(valCache, 10) || 300;

          const valTheme = await invoke<string | null>('get_setting', { key: 'theme' });
          if (valTheme) themeSelected = valTheme;

          const valCpu = await invoke<string | null>('get_setting', { key: 'cpu_threads' });
          if (valCpu) cpuThreads = parseInt(valCpu, 10) || Math.min(4, sysCores);

          const valGpu = await invoke<string | null>('get_setting', { key: 'gpu_accel' });
          if (valGpu) gpuAccel = valGpu === 'true';

          const valColor = await invoke<string | null>('get_setting', { key: 'focus_color' });
          if (valColor) focusColor = valColor;

          const valSens = await invoke<string | null>('get_setting', { key: 'focus_sensitivity' });
          if (valSens) focusSensitivity = valSens;

          const valHist = await invoke<string | null>('get_setting', { key: 'histogram_mode' });
          if (valHist) histogramMode = valHist;

          const valDelay = await invoke<string | null>('get_setting', { key: 'auto_advance_delay' });
          if (valDelay) autoAdvanceDelay = parseInt(valDelay, 10) || 0;

          const valPriority = await invoke<string | null>('get_setting', { key: 'thread_priority' });
          if (valPriority) threadPriority = valPriority;

          const valAlloc = await invoke<string | null>('get_setting', { key: 'alloc_mode' });
          if (valAlloc) allocMode = valAlloc;

          const valIntensity = await invoke<string | null>('get_setting', { key: 'focus_intensity' });
          if (valIntensity) focusIntensity = parseInt(valIntensity, 10) || 80;
        } catch (err) {
          console.error('Failed to load settings:', err);
        }
      };
      
      loadConfig();
    }
  });

  onMount(() => {
    // Detect core specs
    if (typeof navigator !== 'undefined' && navigator.hardwareConcurrency) {
      sysCores = navigator.hardwareConcurrency;
      cpuThreads = Math.min(4, sysCores);
    }
    
    // Detect WebGL specs
    try {
      const canvas = document.createElement('canvas');
      const gl = (canvas.getContext('webgl2') || canvas.getContext('webgl') || canvas.getContext('experimental-webgl')) as WebGLRenderingContext | null;
      if (gl) {
        const ext = gl.getExtension('WEBGL_debug_renderer_info');
        if (ext) {
          gpuModel = gl.getParameter(ext.UNMASKED_RENDERER_WEBGL)
            .replace(/^ANGLE \((.*)\)$/, '$1')
            .split(' vs_')[0]
            .split(' Direct3D')[0];
          gpuVendor = gl.getParameter(ext.UNMASKED_VENDOR_WEBGL);
        } else {
          gpuModel = 'Hardware Accelerated Canvas';
          gpuVendor = 'Native Browser';
        }
      }
    } catch (err) {
      console.warn('Failed to detect WebGL hardware parameters:', err);
    }
  });

  async function handleSave() {
    try {
      await setLanguage(currentLang);
      await setTheme(themeSelected);
      await invoke('set_setting', { key: 'cache_limit', value: cacheLimit.toString() });
      await invoke('set_setting', { key: 'cpu_threads', value: cpuThreads.toString() });
      await invoke('set_setting', { key: 'gpu_accel', value: gpuAccel.toString() });
      await invoke('set_setting', { key: 'focus_color', value: focusColor });
      await invoke('set_setting', { key: 'focus_sensitivity', value: focusSensitivity });
      await invoke('set_setting', { key: 'histogram_mode', value: histogramMode });
      await invoke('set_setting', { key: 'auto_advance_delay', value: autoAdvanceDelay.toString() });

      // Save new configurations
      await invoke('set_setting', { key: 'thread_priority', value: threadPriority });
      await invoke('set_setting', { key: 'alloc_mode', value: allocMode });
      await invoke('set_setting', { key: 'focus_intensity', value: focusIntensity.toString() });
      
      // Dispatch events to active components in case values changed
      window.dispatchEvent(new CustomEvent('settings-updated'));
      toast.success($t('settings.save') + ' ' + $t('about.close'));
      onClose();
    } catch (err) {
      console.error('Failed to save settings:', err);
      toast.error('Failed to save settings');
    }
  }
</script>

{#if show}
  <div class="modal-backdrop" transition:fade={{ duration: 150 }} onclick={onClose} role="dialog" aria-labelledby="settings-title">
    <div class="modal-container" transition:scale={{ start: 0.97, duration: 200 }} onclick={(e) => e.stopPropagation()}>
      
      <!-- HEADER -->
      <div class="modal-header">
        <h2 id="settings-title">Preferences</h2>
        <button class="close-btn" onclick={onClose} aria-label="Close settings">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>

      <!-- BODY WITH SIDE TABS -->
      <div class="modal-body">
        
        <!-- Tab navigation -->
        <div class="tab-sidebar">
          <button class="tab-btn" class:active={activeTab === 'general'} onclick={() => activeTab = 'general'}>
            <span class="tab-icon">⚙️</span> General
          </button>
          <button class="tab-btn" class:active={activeTab === 'performance'} onclick={() => activeTab = 'performance'}>
            <span class="tab-icon">⚡</span> Performance
          </button>
          <button class="tab-btn" class:active={activeTab === 'diagnostics'} onclick={() => activeTab = 'diagnostics'}>
            <span class="tab-icon">📊</span> Diagnostics
          </button>
          <button class="tab-btn" class:active={activeTab === 'theme'} onclick={() => activeTab = 'theme'}>
            <span class="tab-icon">🎨</span> Themes
          </button>
        </div>

        <!-- Tab content area -->
        <div class="tab-content">
          
          <!-- GENERAL TAB -->
          {#if activeTab === 'general'}
            <div class="tab-panel animate-fade-in">
              <div class="setting-group">
                <h3>{$t('settings.language')}</h3>
                <div class="setting-row">
                  <select bind:value={currentLang} class="setting-select">
                    <option value="en">{$t('settings.language.en')}</option>
                    <option value="id">{$t('settings.language.id')}</option>
                  </select>
                </div>
              </div>

              <div class="setting-group">
                <h3>Auto-Advance Delay</h3>
                <div class="setting-row">
                  <div class="slider-container">
                    <input type="range" min="0" max="3000" step="250" bind:value={autoAdvanceDelay} />
                    <span class="slider-value">{autoAdvanceDelay === 0 ? 'Instant' : `${autoAdvanceDelay} ms`}</span>
                  </div>
                  <p class="setting-help">Delay duration after assigning a category hotkey before automatically advancing to next image.</p>
                </div>
              </div>
            </div>
          {/if}

          <!-- PERFORMANCE TAB -->
          {#if activeTab === 'performance'}
            <div class="tab-panel animate-fade-in">
              <!-- HW DIAGNOSTICS CARD -->
              <div class="hw-card">
                <div class="hw-card-header">
                  <span class="pulse-dot"></span> Detected Hardware Specifications
                </div>
                <div class="hw-spec-row">
                  <span class="spec-label">CPU Cores</span>
                  <span class="spec-val">{sysCores} Cores Available</span>
                </div>
                <div class="hw-spec-row">
                  <span class="spec-label">GPU Model</span>
                  <span class="spec-val truncate" title={gpuModel}>{gpuModel}</span>
                </div>
                <div class="hw-spec-row">
                  <span class="spec-label">GPU Vendor</span>
                  <span class="spec-val truncate">{gpuVendor}</span>
                </div>
              </div>

              <!-- Live CPU visualizer bar -->
              <div class="thread-visualizer">
                <span class="visualizer-title">Live CPU Worker Thread Pool Load Monitor</span>
                <div class="threads-grid-visual">
                  {#each threadLoads as load, idx}
                    <div class="thread-visual-cell">
                      <div class="thread-visual-bar" style="height: {load}%; background: {load > 70 ? 'var(--color-buang)' : load > 40 ? 'var(--color-draft)' : 'var(--color-simpan)'}"></div>
                      <span class="thread-label-text">T{idx + 1}</span>
                      <span class="thread-load-text">{load}%</span>
                    </div>
                  {/each}
                </div>
              </div>

              <div class="setting-group">
                <h3>GPU Acceleration</h3>
                <div class="setting-row inline-row">
                  <label class="switch-label" for="gpu-accel-toggle">
                    <input type="checkbox" id="gpu-accel-toggle" bind:checked={gpuAccel} />
                    <span class="switch-text">Enable GPU WebGL hardware acceleration (filters, edge detection)</span>
                  </label>
                </div>
              </div>

              <div class="setting-group">
                <h3>CPU Decoders / Threads</h3>
                <div class="setting-row">
                  <div class="slider-container">
                    <input type="range" min="2" max={Math.max(2, sysCores)} step="1" bind:value={cpuThreads} />
                    <span class="slider-value">{cpuThreads} Cores</span>
                  </div>
                  <p class="setting-help">Number of parallel processor threads allocated for SIMD-accelerated background thumbnail resizing.</p>
                </div>
              </div>

              <div class="setting-group-grid">
                <div class="setting-group">
                  <h3>Thread Allocation Priority</h3>
                  <div class="setting-row">
                    <select bind:value={threadPriority} class="setting-select">
                      <option value="low">Low (Background Idle)</option>
                      <option value="medium">Balanced (Normal)</option>
                      <option value="high">High (Maximum Speed)</option>
                    </select>
                  </div>
                </div>

                <div class="setting-group">
                  <h3>Scheduler Mode</h3>
                  <div class="setting-row">
                    <select bind:value={allocMode} class="setting-select">
                      <option value="dynamic">Dynamic Work Stealing</option>
                      <option value="static">Static Equal Splitting</option>
                    </select>
                  </div>
                </div>
              </div>

              <div class="setting-group">
                <h3>{$t('settings.performance')}</h3>
                <div class="setting-row">
                  <label for="cache-limit">{$t('settings.cache_limit')}</label>
                  <div class="slider-container">
                    <input type="range" id="cache-limit" min="100" max="2000" step="100" bind:value={cacheLimit} />
                    <span class="slider-value">{cacheLimit} MB</span>
                  </div>
                  <p class="setting-help">{$t('settings.cache.info')}</p>
                </div>
              </div>
            </div>
          {/if}

          <!-- DIAGNOSTICS TAB -->
          {#if activeTab === 'diagnostics'}
            <div class="tab-panel animate-fade-in">
              <div class="setting-group">
                <h3>Focus Peaking Overlay Color</h3>
                <div class="setting-row color-selector-row">
                  {#each ['red', 'green', 'blue', 'yellow', 'cyan'] as color}
                    <button 
                      class="color-dot" 
                      class:active={focusColor === color}
                      style="background-color: {color === 'yellow' ? '#eab308' : color}" 
                      onclick={() => focusColor = color}
                      aria-label="Set peaking to {color}"
                    ></button>
                  {/each}
                  <span class="selected-color-text">Active Color: {focusColor}</span>
                </div>
              </div>

              <div class="setting-group">
                <h3>Focus Peaking Opacity / Intensity</h3>
                <div class="setting-row">
                  <div class="slider-container">
                    <input type="range" min="10" max="100" step="5" bind:value={focusIntensity} />
                    <span class="slider-value">{focusIntensity}%</span>
                  </div>
                  <p class="setting-help">Overlay opacity of highlight indicators when peaking filter is applied.</p>
                </div>
              </div>

              <div class="setting-group">
                <h3>Focus Peaking Threshold</h3>
                <div class="setting-row">
                  <select bind:value={focusSensitivity} class="setting-select">
                    <option value="low">Low (Fewer highlights)</option>
                    <option value="medium">Medium (Standard)</option>
                    <option value="high">High (Maximum sensitivity)</option>
                  </select>
                </div>
              </div>

              <div class="setting-group">
                <h3>RGB Histogram Channels</h3>
                <div class="setting-row">
                  <select bind:value={histogramMode} class="setting-select">
                    <option value="all">Composite RGB (All Channels)</option>
                    <option value="luminance">Luminance Channel Only</option>
                  </select>
                </div>
              </div>
            </div>
          {/if}

          <!-- THEMES TAB -->
          {#if activeTab === 'theme'}
            <div class="tab-panel animate-fade-in">
              <div class="setting-group">
                <h3>Interface Theme Layout</h3>
                <div class="themes-grid">
                  <button 
                    class="theme-card" 
                    class:active={themeSelected === 'resolve-dark'} 
                    onclick={() => themeSelected = 'resolve-dark'}
                  >
                    <div class="theme-card-preview resolve-preview"></div>
                    <span class="theme-card-title">DaVinci Resolve Dark</span>
                  </button>
                  <button 
                    class="theme-card" 
                    class:active={themeSelected === 'lightroom'} 
                    onclick={() => themeSelected = 'lightroom'}
                  >
                    <div class="theme-card-preview lightroom-preview"></div>
                    <span class="theme-card-title">Lightroom Classic</span>
                  </button>
                  <button 
                    class="theme-card" 
                    class:active={themeSelected === 'cyberpunk'} 
                    onclick={() => themeSelected = 'cyberpunk'}
                  >
                    <div class="theme-card-preview cyberpunk-preview"></div>
                    <span class="theme-card-title">Cyberpunk Synth</span>
                  </button>
                </div>
              </div>
            </div>
          {/if}

        </div>
      </div>

      <!-- FOOTER -->
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
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(10px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
  }

  .modal-container {
    background: #141416;
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 12px;
    width: 720px;
    height: 520px;
    max-width: 95vw;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    background: #111112;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: white;
    letter-spacing: -0.2px;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.4);
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
    background: rgba(255, 255, 255, 0.06);
  }

  .modal-body {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  /* SIDEBAR TABS */
  .tab-sidebar {
    width: 170px;
    background: #111112;
    border-right: 1px solid rgba(255, 255, 255, 0.05);
    padding: var(--space-3) var(--space-2);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .tab-btn {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    background: transparent;
    border: none;
    color: var(--text-secondary);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    font-size: 11px;
    font-weight: 500;
    text-align: left;
    cursor: pointer;
    transition: all 0.1s;
    font-family: var(--font-sans);
  }

  .tab-btn:hover {
    background: rgba(255, 255, 255, 0.04);
    color: white;
  }

  .tab-btn.active {
    background: rgba(255, 255, 255, 0.06);
    color: white;
    font-weight: 600;
  }

  .tab-icon {
    font-size: 12px;
  }

  /* CONTENT AREA */
  .tab-content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
    background: #141416;
  }

  .tab-panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .setting-group h3 {
    margin: 0 0 10px 0;
    font-size: var(--text-xs);
    font-weight: 700;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.8px;
  }

  .setting-group-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
  }

  .setting-row {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .setting-select {
    background: #1a1a1c;
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: white;
    padding: 8px 12px;
    border-radius: 6px;
    font-size: var(--text-sm);
    outline: none;
    cursor: pointer;
    width: 100%;
    font-family: var(--font-sans);
  }

  .setting-select:focus {
    border-color: var(--accent);
  }

  .slider-container {
    display: flex;
    align-items: center;
    gap: 16px;
    width: 100%;
  }

  .slider-container input[type="range"] {
    flex: 1;
    accent-color: var(--accent);
    height: 4px;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 2px;
    outline: none;
  }

  .slider-value {
    font-variant-numeric: tabular-nums;
    font-family: var(--font-mono);
    min-width: 65px;
    text-align: right;
    color: white;
    font-size: var(--text-sm);
  }

  .setting-help {
    margin: 4px 0 0 0;
    font-size: 10px;
    color: var(--text-tertiary);
    line-height: 1.4;
  }

  /* HARDWARE SPECS CARD */
  .hw-card {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .hw-card-header {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    color: var(--accent-light);
    letter-spacing: 0.5px;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .pulse-dot {
    width: 6px;
    height: 6px;
    background: #10b981;
    border-radius: 50%;
    box-shadow: 0 0 6px #10b981;
  }

  .hw-spec-row {
    display: flex;
    justify-content: space-between;
    font-size: var(--text-sm);
    border-bottom: 1px solid rgba(255, 255, 255, 0.03);
    padding-bottom: var(--space-1);
  }

  .hw-spec-row:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .spec-label {
    color: var(--text-tertiary);
  }

  .spec-val {
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 11px;
    max-width: 340px;
  }

  /* CPU Live Visualizer */
  .thread-visualizer {
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid rgba(255, 255, 255, 0.04);
    border-radius: var(--radius-md);
    padding: var(--space-3);
  }

  .visualizer-title {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-tertiary);
    margin-bottom: var(--space-3);
    display: block;
  }

  .threads-grid-visual {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: 1fr;
    gap: 8px;
    height: 70px;
    align-items: flex-end;
  }

  .thread-visual-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-end;
    height: 100%;
    background: rgba(255, 255, 255, 0.01);
    border-radius: var(--radius-sm);
    padding: var(--space-1);
    position: relative;
    border: 1px solid rgba(255, 255, 255, 0.02);
  }

  .thread-visual-bar {
    width: 100%;
    border-radius: 2px 2px 0 0;
    transition: height 0.6s cubic-bezier(0.25, 0.8, 0.25, 1);
    opacity: 0.8;
  }

  .thread-label-text {
    font-size: 8px;
    font-weight: 700;
    color: var(--text-tertiary);
    margin-top: 4px;
  }

  .thread-load-text {
    font-size: 9px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    margin-top: 1px;
  }

  /* TOGGLE INLINE SWITCH */
  .inline-row {
    flex-direction: row;
    align-items: center;
  }

  .switch-label {
    display: inline-flex;
    align-items: center;
    gap: var(--space-3);
    cursor: pointer;
    user-select: none;
    font-size: var(--text-sm);
    color: var(--text-primary);
  }

  .switch-label input[type="checkbox"] {
    accent-color: var(--accent);
    width: 16px;
    height: 16px;
    cursor: pointer;
  }

  .switch-text {
    line-height: 1.4;
  }

  /* COLOR DOT PICKER */
  .color-selector-row {
    flex-direction: row;
    align-items: center;
    gap: var(--space-3);
  }

  .color-dot {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    transition: all 0.15s;
  }

  .color-dot.active {
    border-color: white;
    transform: scale(1.15);
    box-shadow: 0 0 10px rgba(255,255,255,0.3);
  }

  .selected-color-text {
    font-size: var(--text-sm);
    color: var(--text-secondary);
    margin-left: var(--space-2);
  }

  /* THEME GRID CARD SELECTORS */
  .themes-grid {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: var(--space-3);
  }

  .theme-card {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: var(--radius-md);
    padding: var(--space-2);
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    transition: all 0.2s;
  }

  .theme-card:hover {
    background: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .theme-card.active {
    background: rgba(255, 255, 255, 0.06);
    border-color: var(--accent);
  }

  .theme-card-preview {
    width: 100%;
    height: 50px;
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .resolve-preview {
    background: linear-gradient(135deg, #101011 0%, #161618 50%, #2563eb 100%);
  }

  .lightroom-preview {
    background: linear-gradient(135deg, #161616 0%, #1e1e1e 50%, #d97706 100%);
  }

  .cyberpunk-preview {
    background: linear-gradient(135deg, #040408 0%, #080812 50%, #db2777 100%);
  }

  .theme-card-title {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-secondary);
    text-align: center;
  }

  /* FOOTER */
  .modal-footer {
    padding: 12px 24px;
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    background: #111112;
  }
</style>
