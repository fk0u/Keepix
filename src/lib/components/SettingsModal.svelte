<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { t, locale, setLanguage, type Language } from '$lib/i18n';
  import { invoke } from '@tauri-apps/api/core';
  import { activeTheme, setTheme } from '$lib/stores/ui';
  import { onMount, onDestroy } from 'svelte';
  import { toast } from '$lib/stores/toast';

  let { show = false, onClose }: { show?: boolean; onClose: () => void } = $props();

  // Settings State
  let activeTab = $state<'general' | 'performance' | 'ai' | 'diagnostics' | 'theme'>('general');
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

  // Caching Stats
  let currentJsCacheSize = $state(0);

  // General Preferences
  let defaultViewMode = $state('grid');
  let autoAdvanceCategory = $state(true);
  let autoAdvanceRating = $state(true);
  let autoAdvanceLabel = $state(true);
  let defaultGridSize = $state(200);
  let syncZoomDefault = $state(true);

  // AI & VLM Preferences
  let enableVlm = $state(false);
  let vlmProvider = $state<'ollama' | 'nvidia_nim'>('ollama');
  let ollamaUrl = $state('http://localhost:11434');
  let ollamaModel = $state('llava');
  let ollamaStatus = $state<'idle' | 'testing' | 'online' | 'offline'>('idle');

  let nvidiaApiKey = $state('');
  let nvidiaModel = $state('moonshotai/kimi-k2.6');
  let nvidiaStatus = $state<'idle' | 'testing' | 'online' | 'offline'>('idle');

  let aestheticWeight = $state(0.40);
  let technicalWeight = $state(0.40);
  let faceWeight = $state(0.20);
  let duplicateThreshold = $state(0.92);

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

          // General settings
          const valView = await invoke<string | null>('get_setting', { key: 'default_view_mode' });
          if (valView) defaultViewMode = valView;
          
          const valAdvCat = await invoke<string | null>('get_setting', { key: 'auto_advance_category' });
          if (valAdvCat) autoAdvanceCategory = valAdvCat === 'true';

          const valAdvRat = await invoke<string | null>('get_setting', { key: 'auto_advance_rating' });
          if (valAdvRat) autoAdvanceRating = valAdvRat === 'true';

          const valAdvLab = await invoke<string | null>('get_setting', { key: 'auto_advance_label' });
          if (valAdvLab) autoAdvanceLabel = valAdvLab === 'true';

          const valGridSize = await invoke<string | null>('get_setting', { key: 'default_grid_size' });
          if (valGridSize) defaultGridSize = parseInt(valGridSize, 10) || 200;

          const valSyncZoom = await invoke<string | null>('get_setting', { key: 'sync_zoom_default' });
          if (valSyncZoom) syncZoomDefault = valSyncZoom === 'true';

          // Load VLM values from localStorage
          const savedEnableVlm = localStorage.getItem('keepix_enable_vlm');
          if (savedEnableVlm) enableVlm = savedEnableVlm === 'true';
          vlmProvider = (localStorage.getItem('keepix_vlm_provider') as any) || 'ollama';
          ollamaUrl = localStorage.getItem('keepix_ollama_url') || 'http://localhost:11434';
          ollamaModel = localStorage.getItem('keepix_ollama_model') || 'llava';
          nvidiaApiKey = localStorage.getItem('keepix_nvidia_api_key') || '';
          nvidiaModel = localStorage.getItem('keepix_nvidia_model') || 'moonshotai/kimi-k2.6';
          
          // Load AI weights
          const aesW = localStorage.getItem('keepix_default_aesthetic_weight');
          if (aesW) aestheticWeight = parseFloat(aesW);
          const techW = localStorage.getItem('keepix_default_technical_weight');
          if (techW) technicalWeight = parseFloat(techW);
          const faceW = localStorage.getItem('keepix_default_face_weight');
          if (faceW) faceWeight = parseFloat(faceW);
          const dupT = localStorage.getItem('keepix_default_duplicate_threshold');
          if (dupT) duplicateThreshold = parseFloat(dupT);

          // Get active JS cache size
          import('$lib/services/image-cache').then(({ getCacheSize }) => {
            currentJsCacheSize = getCacheSize();
          });
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

  async function testOllamaConnection() {
    ollamaStatus = 'testing';
    try {
      const res = await fetch(`${ollamaUrl}/api/tags`);
      if (res.ok) {
        ollamaStatus = 'online';
        toast.success('Ollama Neural Node is online!');
      } else {
        ollamaStatus = 'offline';
        toast.error('Ollama returned an error status.');
      }
    } catch (err) {
      ollamaStatus = 'offline';
      toast.error('Could not connect to Ollama. Make sure it is running.');
    }
  }

  async function testNvidiaConnection() {
    if (!nvidiaApiKey) {
      toast.error('Please enter your NVIDIA API Key first.');
      return;
    }
    nvidiaStatus = 'testing';
    try {
      const dummyGif = 'R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
      const resp = await invoke<string>('query_nvidia_nim_vision', {
        imageBase64: dummyGif,
        model: nvidiaModel,
        prompt: 'Say Connected',
        apiKey: nvidiaApiKey
      });
      if (resp) {
        nvidiaStatus = 'online';
        toast.success(`NVIDIA NIM connected! Response: ${resp.trim()}`);
      } else {
        nvidiaStatus = 'offline';
        toast.error('NVIDIA NIM returned an empty response.');
      }
    } catch (err: any) {
      nvidiaStatus = 'offline';
      toast.error(`NVIDIA NIM connection failed: ${err.message || err}`);
    }
  }

  async function handleClearCache() {
    try {
      import('$lib/services/image-cache').then(async ({ clearImageCache }) => {
        await clearImageCache();
        currentJsCacheSize = 0;
        toast.success($t('settings.cache.clear_success'));
      });
    } catch (err) {
      console.error('Failed to clear image cache:', err);
    }
  }

  async function handleSave() {
    try {
      await setLanguage(currentLang);
      await setTheme(themeSelected);
      await invoke('set_setting', { key: 'cache_limit', value: cacheLimit.toString() });
      
      // Dynamic Rust cache resize command
      await invoke('update_cache_capacity', { capacity: cacheLimit });

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

      // Save general settings
      await invoke('set_setting', { key: 'default_view_mode', value: defaultViewMode });
      await invoke('set_setting', { key: 'auto_advance_category', value: autoAdvanceCategory.toString() });
      await invoke('set_setting', { key: 'auto_advance_rating', value: autoAdvanceRating.toString() });
      await invoke('set_setting', { key: 'auto_advance_label', value: autoAdvanceLabel.toString() });
      await invoke('set_setting', { key: 'default_grid_size', value: defaultGridSize.toString() });
      await invoke('set_setting', { key: 'sync_zoom_default', value: syncZoomDefault.toString() });

      // Save VLM values to localStorage
      localStorage.setItem('keepix_enable_vlm', String(enableVlm));
      localStorage.setItem('keepix_vlm_provider', vlmProvider);
      localStorage.setItem('keepix_ollama_url', ollamaUrl);
      localStorage.setItem('keepix_ollama_model', ollamaModel);
      localStorage.setItem('keepix_nvidia_api_key', nvidiaApiKey);
      localStorage.setItem('keepix_nvidia_model', nvidiaModel);

      // Save AI culling defaults
      localStorage.setItem('keepix_default_aesthetic_weight', aestheticWeight.toString());
      localStorage.setItem('keepix_default_technical_weight', technicalWeight.toString());
      localStorage.setItem('keepix_default_face_weight', faceWeight.toString());
      localStorage.setItem('keepix_default_duplicate_threshold', duplicateThreshold.toString());

      // Dispatch events to active components in case values changed
      window.dispatchEvent(new CustomEvent('settings-updated'));
      toast.success($t('settings.save') + ' ' + $t('about.close'));
      onClose();
    } catch (err) {
      console.error('Failed to save settings:', err);
      toast.error($t('settings.toast.fail'));
    }
  }
</script>

{#if show}
  <div class="modal-backdrop" transition:fade={{ duration: 150 }} onclick={onClose} role="dialog" aria-labelledby="settings-title">
    <div class="modal-container" transition:scale={{ start: 0.97, duration: 200 }} onclick={(e) => e.stopPropagation()}>
      
      <!-- HEADER -->
      <div class="modal-header">
        <h2 id="settings-title">{$t('settings.preferences')}</h2>
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
            <span class="tab-icon">⚙️</span> {$t('settings.tab.general')}
          </button>
          <button class="tab-btn" class:active={activeTab === 'performance'} onclick={() => activeTab = 'performance'}>
            <span class="tab-icon">⚡</span> {$t('settings.tab.performance')}
          </button>
          <button class="tab-btn" class:active={activeTab === 'ai'} onclick={() => activeTab = 'ai'}>
            <span class="tab-icon">🤖</span> {$t('settings.tab.ai')}
          </button>
          <button class="tab-btn" class:active={activeTab === 'diagnostics'} onclick={() => activeTab = 'diagnostics'}>
            <span class="tab-icon">📊</span> {$t('settings.tab.diagnostics')}
          </button>
          <button class="tab-btn" class:active={activeTab === 'theme'} onclick={() => activeTab = 'theme'}>
            <span class="tab-icon">🎨</span> {$t('settings.tab.theme')}
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

              <div class="setting-group-grid">
                <div class="setting-group">
                  <h3>{$t('settings.general.default_view')}</h3>
                  <div class="setting-row">
                    <select bind:value={defaultViewMode} class="setting-select">
                      <option value="grid">{$t('settings.general.default_view.grid')}</option>
                      <option value="preview">{$t('settings.general.default_view.preview')}</option>
                    </select>
                  </div>
                </div>

                <div class="setting-group">
                  <h3>{$t('settings.general.grid_size')}</h3>
                  <div class="setting-row">
                    <div class="slider-container">
                      <input type="range" min="120" max="300" step="10" bind:value={defaultGridSize} />
                      <span class="slider-value">{defaultGridSize} px</span>
                    </div>
                  </div>
                </div>
              </div>

              <div class="setting-group">
                <h3>{$t('settings.general.auto_advance_options')}</h3>
                <div class="setting-row flex-column-checkboxes">
                  <label class="switch-label checkbox-label" for="auto-advance-cat">
                    <input type="checkbox" id="auto-advance-cat" bind:checked={autoAdvanceCategory} />
                    <span class="switch-text">{$t('settings.general.auto_advance_category')}</span>
                  </label>
                  <label class="switch-label checkbox-label" for="auto-advance-rat">
                    <input type="checkbox" id="auto-advance-rat" bind:checked={autoAdvanceRating} />
                    <span class="switch-text">{$t('settings.general.auto_advance_rating')}</span>
                  </label>
                  <label class="switch-label checkbox-label" for="auto-advance-lab">
                    <input type="checkbox" id="auto-advance-lab" bind:checked={autoAdvanceLabel} />
                    <span class="switch-text">{$t('settings.general.auto_advance_label')}</span>
                  </label>
                </div>
              </div>

              <div class="setting-group">
                <h3>{$t('settings.autoadvance_delay')}</h3>
                <div class="setting-row">
                  <div class="slider-container">
                    <input type="range" min="0" max="3000" step="250" bind:value={autoAdvanceDelay} />
                    <span class="slider-value">{autoAdvanceDelay === 0 ? $t('settings.autoadvance_delay.instant') : `${autoAdvanceDelay} ms`}</span>
                  </div>
                  <p class="setting-help">{$t('settings.autoadvance_delay.help')}</p>
                </div>
              </div>

              <div class="setting-group">
                <div class="setting-row inline-row">
                  <label class="switch-label checkbox-label" for="sync-zoom-default">
                    <input type="checkbox" id="sync-zoom-default" bind:checked={syncZoomDefault} />
                    <span class="switch-text">{$t('settings.general.sync_zoom')}</span>
                  </label>
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
                  <span class="pulse-dot"></span> {$t('settings.hardware_specs')}
                </div>
                <div class="hw-spec-row">
                  <span class="spec-label">{$t('settings.cpu_cores')}</span>
                  <span class="spec-val">{$t('settings.cpu_cores.val', {cores: sysCores.toString()})}</span>
                </div>
                <div class="hw-spec-row">
                  <span class="spec-label">{$t('settings.gpu_model')}</span>
                  <span class="spec-val truncate" title={gpuModel}>{gpuModel}</span>
                </div>
                <div class="hw-spec-row">
                  <span class="spec-label">{$t('settings.gpu_vendor')}</span>
                  <span class="spec-val truncate">{gpuVendor}</span>
                </div>
              </div>

              <!-- Live CPU visualizer bar -->
              <div class="thread-visualizer">
                <span class="visualizer-title">{$t('settings.cpu_load_monitor')}</span>
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

              <!-- CACHE DIAGNOSTICS CARD -->
              <div class="hw-card diagnostic-card">
                <div class="hw-card-header text-accent">
                  <span class="pulse-dot active-dot"></span> {$t('settings.cache.stats')}
                </div>
                <div class="hw-spec-row flex-align-center">
                  <span class="spec-label">{$t('settings.cache.active_entries')}</span>
                  <span class="spec-val">{currentJsCacheSize} Photos</span>
                </div>
                <div class="hw-spec-row inline-row-btn" style="margin-top: var(--space-2)">
                  <button class="btn btn-sm btn-clear-cache" onclick={handleClearCache}>
                    🗑️ {$t('settings.cache.clear_btn')}
                  </button>
                </div>
              </div>

              <div class="setting-group">
                <h3>{$t('settings.gpu_accel')}</h3>
                <div class="setting-row inline-row">
                  <label class="switch-label checkbox-label" for="gpu-accel-toggle">
                    <input type="checkbox" id="gpu-accel-toggle" bind:checked={gpuAccel} />
                    <span class="switch-text">{$t('settings.gpu_accel.help')}</span>
                  </label>
                </div>
              </div>

              <div class="setting-group-grid">
                <div class="setting-group">
                  <h3>{$t('settings.cpu_decoders')}</h3>
                  <div class="setting-row">
                    <div class="slider-container">
                      <input type="range" min="2" max={Math.max(2, sysCores)} step="1" bind:value={cpuThreads} />
                      <span class="slider-value">{cpuThreads} Cores</span>
                    </div>
                    <p class="setting-help">{$t('settings.cpu_decoders.help')}</p>
                  </div>
                </div>

                <div class="setting-group">
                  <h3>{$t('settings.cache_limit')}</h3>
                  <div class="setting-row">
                    <div class="slider-container">
                      <input type="range" id="cache-limit" min="100" max="1000" step="50" bind:value={cacheLimit} />
                      <span class="slider-value">{cacheLimit} Photos</span>
                    </div>
                    <p class="setting-help">{$t('settings.cache_limit.help')}</p>
                  </div>
                </div>
              </div>

              <div class="setting-group-grid">
                <div class="setting-group">
                  <h3>{$t('settings.thread_priority')}</h3>
                  <div class="setting-row">
                    <select bind:value={threadPriority} class="setting-select">
                      <option value="low">{$t('settings.thread_priority.low')}</option>
                      <option value="medium">{$t('settings.thread_priority.medium')}</option>
                      <option value="high">{$t('settings.thread_priority.high')}</option>
                    </select>
                  </div>
                </div>

                <div class="setting-group">
                  <h3>{$t('settings.scheduler_mode')}</h3>
                  <div class="setting-row">
                    <select bind:value={allocMode} class="setting-select">
                      <option value="dynamic">{$t('settings.scheduler_mode.dynamic')}</option>
                      <option value="static">{$t('settings.scheduler_mode.static')}</option>
                    </select>
                  </div>
                </div>
              </div>
            </div>
          {/if}

          <!-- AI & CULLING TAB -->
          {#if activeTab === 'ai'}
            <div class="tab-panel animate-fade-in">
              <div class="setting-group">
                <h3>{$t('settings.ai.default_weights')}</h3>
                <div class="setting-row slider-row-group">
                  <div class="weight-slider-item">
                    <span class="weight-label">{$t('settings.ai.aesthetic')}</span>
                    <div class="slider-container">
                      <input type="range" min="0" max="1" step="0.05" bind:value={aestheticWeight} />
                      <span class="slider-value">{Math.round(aestheticWeight * 100)}%</span>
                    </div>
                  </div>
                  <div class="weight-slider-item">
                    <span class="weight-label">{$t('settings.ai.technical')}</span>
                    <div class="slider-container">
                      <input type="range" min="0" max="1" step="0.05" bind:value={technicalWeight} />
                      <span class="slider-value">{Math.round(technicalWeight * 100)}%</span>
                    </div>
                  </div>
                  <div class="weight-slider-item">
                    <span class="weight-label">{$t('settings.ai.face')}</span>
                    <div class="slider-container">
                      <input type="range" min="0" max="1" step="0.05" bind:value={faceWeight} />
                      <span class="slider-value">{Math.round(faceWeight * 100)}%</span>
                    </div>
                  </div>
                </div>
              </div>

              <div class="setting-group">
                <h3>{$t('settings.ai.duplicate')}</h3>
                <div class="setting-row">
                  <div class="slider-container">
                    <input type="range" min="0.80" max="0.99" step="0.01" bind:value={duplicateThreshold} />
                    <span class="slider-value">{Math.round(duplicateThreshold * 100)}%</span>
                  </div>
                </div>
              </div>

              <div class="setting-group border-top-group">
                <h3>{$t('settings.ai.vlm_reasoning')}</h3>
                <div class="setting-row" style="margin-bottom: var(--space-3)">
                  <label class="switch-label checkbox-label" for="enable-vlm-toggle">
                    <input type="checkbox" id="enable-vlm-toggle" bind:checked={enableVlm} />
                    <span class="switch-text">Enable Cognitive Vision & Natural Reasoning Fallback</span>
                  </label>
                </div>

                {#if enableVlm}
                  <div class="vlm-config-panel animate-slide-up">
                    <div class="setting-row select-row" style="margin-bottom: var(--space-3)">
                      <label for="vlm-provider-select">{$t('settings.ai.vlm_provider')}</label>
                      <select id="vlm-provider-select" bind:value={vlmProvider} class="setting-select">
                        <option value="ollama">Local Neural Node (Ollama)</option>
                        <option value="nvidia_nim">Cloud Acceleration (NVIDIA NIM)</option>
                      </select>
                    </div>

                    {#if vlmProvider === 'ollama'}
                      <div class="provider-fields animate-fade-in">
                        <div class="setting-row" style="margin-bottom: var(--space-2)">
                          <label for="ollama-url">{$t('settings.ai.ollama_url')}</label>
                          <input type="text" id="ollama-url" bind:value={ollamaUrl} class="setting-input-text" />
                        </div>
                        <div class="setting-row" style="margin-bottom: var(--space-3)">
                          <label for="ollama-model">{$t('settings.ai.ollama_model')}</label>
                          <input type="text" id="ollama-model" bind:value={ollamaModel} class="setting-input-text" />
                        </div>
                        <div class="setting-row align-btn-status">
                          <button class="btn btn-sm" onclick={testOllamaConnection} disabled={ollamaStatus === 'testing'}>
                            {ollamaStatus === 'testing' ? 'Testing...' : $t('settings.ai.test_connection')}
                          </button>
                          <span class="status-indicator {ollamaStatus}">
                            {ollamaStatus.toUpperCase()}
                          </span>
                        </div>
                      </div>
                    {:else}
                      <div class="provider-fields animate-fade-in">
                        <div class="setting-row" style="margin-bottom: var(--space-2)">
                          <label for="nvidia-key">{$t('settings.ai.nvidia_key')}</label>
                          <input type="password" id="nvidia-key" bind:value={nvidiaApiKey} class="setting-input-text" placeholder="nvapi-..." />
                        </div>
                        <div class="setting-row" style="margin-bottom: var(--space-3)">
                          <label for="nvidia-model">{$t('settings.ai.nvidia_model')}</label>
                          <select id="nvidia-model" bind:value={nvidiaModel} class="setting-select">
                            <option value="moonshotai/kimi-k2.6">Kimi k2.6 Vision (Moonshot)</option>
                            <option value="meta/llama-3.2-11b-vision-instruct">Llama 3.2 11B Vision (Meta)</option>
                            <option value="nvidia/vila-15b">VILA 15B (NVIDIA)</option>
                            <option value="microsoft/phi-3.5-vision-instruct">Phi-3.5 Vision (Microsoft)</option>
                          </select>
                        </div>
                        <div class="setting-row align-btn-status">
                          <button class="btn btn-sm" onclick={testNvidiaConnection} disabled={nvidiaStatus === 'testing'}>
                            {nvidiaStatus === 'testing' ? 'Testing...' : $t('settings.ai.test_connection')}
                          </button>
                          <span class="status-indicator {nvidiaStatus}">
                            {nvidiaStatus.toUpperCase()}
                          </span>
                        </div>
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            </div>
          {/if}

          <!-- DIAGNOSTICS TAB -->
          {#if activeTab === 'diagnostics'}
            <div class="tab-panel animate-fade-in">
              <div class="setting-group">
                <h3>{$t('settings.focus_peaking_color')}</h3>
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
                  <span class="selected-color-text">{$t('settings.active_color', {color: focusColor})}</span>
                </div>
              </div>

              <div class="setting-group">
                <h3>{$t('settings.focus_peaking_intensity')}</h3>
                <div class="setting-row">
                  <div class="slider-container">
                    <input type="range" min="10" max="100" step="5" bind:value={focusIntensity} />
                    <span class="slider-value">{focusIntensity}%</span>
                  </div>
                  <p class="setting-help">{$t('settings.focus_peaking_intensity.help')}</p>
                </div>
              </div>

              <div class="setting-group">
                <h3>{$t('settings.focus_peaking_threshold')}</h3>
                <div class="setting-row">
                  <select bind:value={focusSensitivity} class="setting-select">
                    <option value="low">{$t('settings.focus_peaking_threshold.low')}</option>
                    <option value="medium">{$t('settings.focus_peaking_threshold.medium')}</option>
                    <option value="high">{$t('settings.focus_peaking_threshold.high')}</option>
                  </select>
                </div>
              </div>

              <div class="setting-group">
                <h3>{$t('settings.rgb_histogram')}</h3>
                <div class="setting-row">
                  <select bind:value={histogramMode} class="setting-select">
                    <option value="all">{$t('settings.rgb_histogram.all')}</option>
                    <option value="luminance">{$t('settings.rgb_histogram.luminance')}</option>
                  </select>
                </div>
              </div>
            </div>
          {/if}

          <!-- THEMES TAB -->
          {#if activeTab === 'theme'}
            <div class="tab-panel animate-fade-in">
              <div class="setting-group">
                <h3>{$t('settings.interface_theme')}</h3>
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

  /* ADVANCED PREFERENCES PRESET STYLES */
  .flex-column-checkboxes {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    cursor: pointer;
    font-size: var(--text-sm);
    color: var(--text-secondary);
    transition: color 0.15s;
  }

  .checkbox-label:hover {
    color: white;
  }

  .checkbox-label input[type="checkbox"] {
    accent-color: var(--accent);
    cursor: pointer;
    width: 15px;
    height: 15px;
  }

  .btn-clear-cache {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.25);
    color: #ef4444;
    transition: all 0.2s;
  }

  .btn-clear-cache:hover {
    background: #ef4444;
    color: white;
    border-color: transparent;
    box-shadow: 0 0 10px rgba(239, 68, 68, 0.3);
  }

  .diagnostic-card {
    border-color: var(--accent-soft);
    background: rgba(37, 99, 235, 0.03);
  }

  .text-accent {
    color: var(--accent-light) !important;
  }

  .active-dot {
    background: var(--accent-light);
    box-shadow: 0 0 6px var(--accent-light);
  }

  .border-top-group {
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    padding-top: var(--space-4);
  }

  .vlm-config-panel {
    background: rgba(255, 255, 255, 0.01);
    border: 1px solid rgba(255, 255, 255, 0.04);
    border-radius: var(--radius-md);
    padding: var(--space-3);
    margin-top: var(--space-2);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .setting-input-text {
    background: #1a1a1c;
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: white;
    padding: 8px 12px;
    border-radius: 6px;
    font-size: var(--text-sm);
    outline: none;
    width: 100%;
    font-family: var(--font-sans);
    transition: border-color 0.15s, box-shadow 0.15s;
  }

  .setting-input-text:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }

  .slider-row-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .weight-slider-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .weight-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .align-btn-status {
    flex-direction: row !important;
    align-items: center;
    gap: var(--space-3);
  }

  .status-indicator {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-tertiary);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .status-indicator.online {
    color: #10b981;
    background: rgba(16, 185, 129, 0.1);
    border-color: rgba(16, 185, 129, 0.2);
    box-shadow: 0 0 8px rgba(16, 185, 129, 0.1);
  }

  .status-indicator.offline {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
    border-color: rgba(239, 68, 68, 0.2);
  }

  .status-indicator.testing {
    color: #f59e0b;
    background: rgba(245, 158, 11, 0.1);
    border-color: rgba(245, 158, 11, 0.2);
  }

  .status-indicator.idle {
    color: var(--text-secondary);
    background: rgba(255, 255, 255, 0.03);
    border-color: rgba(255, 255, 255, 0.05);
  }
</style>
