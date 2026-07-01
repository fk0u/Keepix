<script lang="ts">
  import { showAiCull } from '$lib/stores/ui';
  import { mediaItems, setCategoryForItem, setStarRatingForItem, setColorLabelForItem, loadCategoryStats, currentItem, navigateTo } from '$lib/stores/media';
  import { currentProject } from '$lib/stores/project';
  import { get } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { toast } from '$lib/stores/toast';
  import { t, locale } from '$lib/i18n';
  import { onMount } from 'svelte';

  // Config State
  let hasModels = $state(false);
  let isDownloading = $state(false);
  let downloadProgress = $state<Record<string, number>>({
    'clip_image.onnx': 0,
    'laion_aesthetic.onnx': 0,
    'technical_musiq.onnx': 0,
    'face_detector.onnx': 0
  });
  let downloadStatusText = $state('Not Initialized');

  // Weights & Thresholds
  let aestheticWeight = $state(0.40);
  let technicalWeight = $state(0.40);
  let faceWeight = $state(0.20);
  let duplicateThreshold = $state(0.92);

  // Learn My Style
  let knnConfidenceThreshold = $state(0.80);
  let styleDatasetSize = $state(0);

  // VLM Settings
  let enableVlm = $state(false);
  let vlmProvider = $state<'ollama' | 'nvidia_nim' | 'gemini'>('ollama');
  let ollamaUrl = $state('http://localhost:11434');
  let ollamaModel = $state('llava');
  let ollamaStatus = $state<'idle' | 'testing' | 'online' | 'offline'>('idle');

  let nvidiaApiKey = $state((import.meta.env as any).VITE_NVIDIA_API_KEY || '');
  let nvidiaModel = $state('moonshotai/kimi-k2.6');
  let nvidiaStatus = $state<'idle' | 'testing' | 'online' | 'offline'>('idle');

  let geminiApiKey = $state('');
  let geminiModel = $state('gemini-2.5-flash');
  let geminiStatus = $state<'idle' | 'testing' | 'online' | 'offline'>('idle');
  let enableStructuredCulling = $state(false);

  // Culling Session State
  let isCulling = $state(false);
  let cullingPhase = $state<'loading' | 'scoring' | 'duplicates' | 'complete' | 'idle'>('idle');
  let currentFile = $state('');
  let currentAesthetic = $state(0.0);
  let currentTechnical = $state(0.0);
  let currentOverall = $state(0.0);
  let currentFaceCount = $state(0);
  let currentBlinkDetected = $state(false);
  let totalPhotos = $state(0);
  let processedPhotos = $state(0);
  let consoleLogs = $state<{ text: string; type: 'info' | 'success' | 'warn' | 'error' }[]>([]);
  let logsContainer = $state<HTMLDivElement>();

  // Statistics
  let stats = $state({
    best: 0,
    trash: 0,
    review: 0,
    duplicates: 0
  });

  function loadLocalSettings() {
    const savedEnableVlm = localStorage.getItem('keepix_enable_vlm');
    if (savedEnableVlm) enableVlm = savedEnableVlm === 'true';
    vlmProvider = (localStorage.getItem('keepix_vlm_provider') as any) || 'ollama';
    ollamaUrl = localStorage.getItem('keepix_ollama_url') || 'http://localhost:11434';
    ollamaModel = localStorage.getItem('keepix_ollama_model') || 'llava';
    nvidiaApiKey = localStorage.getItem('keepix_nvidia_api_key') || (import.meta.env as any).VITE_NVIDIA_API_KEY || '';
    nvidiaModel = localStorage.getItem('keepix_nvidia_model') || 'moonshotai/kimi-k2.6';

    geminiApiKey = localStorage.getItem('keepix_gemini_api_key') || '';
    geminiModel = localStorage.getItem('keepix_gemini_model') || 'gemini-2.5-flash';
    const savedEnableStructuredCulling = localStorage.getItem('keepix_enable_structured_culling');
    if (savedEnableStructuredCulling) enableStructuredCulling = savedEnableStructuredCulling === 'true';

    const aesW = localStorage.getItem('keepix_default_aesthetic_weight');
    if (aesW) aestheticWeight = parseFloat(aesW);
    const techW = localStorage.getItem('keepix_default_technical_weight');
    if (techW) technicalWeight = parseFloat(techW);
    const faceW = localStorage.getItem('keepix_default_face_weight');
    if (faceW) faceWeight = parseFloat(faceW);
    const dupT = localStorage.getItem('keepix_default_duplicate_threshold');
    if (dupT) duplicateThreshold = parseFloat(dupT);
  }

  onMount(() => {
    // Check if models exist
    invoke<boolean>('check_ai_models_status')
      .then((res) => {
        hasModels = res;
        if (!hasModels) {
          // Automatically start download of standard model weights
          startDownloadingModels();
        }
      })
      .catch((e) => {
        console.error(e);
      });

    loadLocalSettings();
    window.addEventListener('settings-updated', loadLocalSettings);
    
    // Fetch manual dataset size (Best vs Trash counts)
    updateStyleDatasetStats();

    return () => {
      window.removeEventListener('settings-updated', loadLocalSettings);
    };
  });

  $effect(() => {
    localStorage.setItem('keepix_enable_vlm', String(enableVlm));
  });
  $effect(() => {
    localStorage.setItem('keepix_vlm_provider', vlmProvider);
  });
  $effect(() => {
    localStorage.setItem('keepix_ollama_url', ollamaUrl);
  });
  $effect(() => {
    localStorage.setItem('keepix_ollama_model', ollamaModel);
  });
  $effect(() => {
    localStorage.setItem('keepix_nvidia_api_key', nvidiaApiKey);
  });
  $effect(() => {
    localStorage.setItem('keepix_nvidia_model', nvidiaModel);
  });
  $effect(() => {
    localStorage.setItem('keepix_gemini_api_key', geminiApiKey);
  });
  $effect(() => {
    localStorage.setItem('keepix_gemini_model', geminiModel);
  });
  $effect(() => {
    localStorage.setItem('keepix_enable_structured_culling', String(enableStructuredCulling));
  });

  async function updateStyleDatasetStats() {
    const proj = get(currentProject);
    if (!proj) return;
    try {
      // We can count items in the media store that are Best (category_id = 2) or Trash (category_id = 1)
      const list = get(mediaItems);
      styleDatasetSize = list.filter(item => item.category_id === 1 || item.category_id === 2).length;
    } catch (e) {
      console.error(e);
    }
  }

  function addLog(text: string, type: 'info' | 'success' | 'warn' | 'error' = 'info') {
    consoleLogs = [...consoleLogs, { text, type }];
    setTimeout(() => {
      if (logsContainer) {
        logsContainer.scrollTop = logsContainer.scrollHeight;
      }
    }, 50);
  }

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
        prompt: 'Say exactly: Connected',
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

  async function testGeminiConnection() {
    if (!geminiApiKey) {
      toast.error('Please enter your Gemini API Key first.');
      return;
    }
    geminiStatus = 'testing';
    try {
      const dummyGif = 'R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
      const resp = await invoke<string>('query_gemini_vision', {
        imageBase64: dummyGif,
        model: geminiModel,
        prompt: 'Say exactly: Connected',
        apiKey: geminiApiKey
      });
      if (resp) {
        geminiStatus = 'online';
        toast.success(`Gemini connected! Response: ${resp.trim()}`);
      } else {
        geminiStatus = 'offline';
        toast.error('Gemini returned an empty response.');
      }
    } catch (err: any) {
      geminiStatus = 'offline';
      toast.error(`Gemini connection failed: ${err.message || err}`);
    }
  }

  function parseJsonFromText(text: string) {
    const match = text.match(/{[\s\S]*}/);
    if (!match) throw new Error("No JSON object found in response");
    return JSON.parse(match[0]);
  }

  async function startDownloadingModels() {
    isDownloading = true;
    downloadStatusText = 'Requesting model stream...';

    const unlistenProgress = await listen<any>('download-progress', (event) => {
      const p = event.payload;
      downloadProgress = {
        ...downloadProgress,
        [p.model_id]: p.percentage
      };
      downloadStatusText = `Downloading ${p.model_id}: ${p.percentage.toFixed(1)}%`;
    });

    const unlistenComplete = await listen<any>('download-complete', (event) => {
      isDownloading = false;
      unlistenProgress();
      unlistenComplete();

      if (event.payload === null || event.payload === undefined) {
        hasModels = true;
        downloadStatusText = 'Ready';
        toast.success('All models successfully stored locally!');
      } else {
        toast.error(`Download failed: ${event.payload}`);
      }
    });

    try {
      await invoke('download_ai_models');
    } catch (err: any) {
      isDownloading = false;
      unlistenProgress();
      unlistenComplete();
      toast.error(`Download request failed: ${err.message}`);
    }
  }

  async function executeLocalCulling() {
    const proj = get(currentProject);
    if (!proj) {
      toast.error('No project is currently open!');
      return;
    }

    isCulling = true;
    consoleLogs = [];
    stats = { best: 0, trash: 0, review: 0, duplicates: 0 };
    processedPhotos = 0;
    
    addLog('Initializing local neural pipeline...', 'info');

    const unlistenAi = await listen<any>('ai-progress', async (event) => {
      const p = event.payload;
      cullingPhase = p.phase;
      totalPhotos = p.total;
      processedPhotos = p.current;
      currentFile = p.current_file;
      currentAesthetic = p.aesthetic_score;
      currentTechnical = p.technical_score;
      currentOverall = p.overall_score;
      currentFaceCount = p.face_count;
      currentBlinkDetected = p.blink_detected;

      if (p.phase === 'loading') {
        addLog(`[SYSTEM] ${p.message}`, 'info');
      } else if (p.phase === 'scoring') {
        addLog(`[SCORER] Analyzed ${p.current_file}: Aesthetic ${p.aesthetic_score.toFixed(1)}/10 | Sharpness ${p.technical_score.toFixed(0)}/100 | Faces: ${p.face_count}`, 'info');
        
        // Live stats update
        let initialCat: 'best' | 'trash' | 'review' = 'review';
        if (p.overall_score > 0.75) {
          stats.best++;
          initialCat = 'best';
        } else if (p.overall_score < 0.25) {
          stats.trash++;
          initialCat = 'trash';
        } else {
          stats.review++;
          initialCat = 'review';
        }

        // Optional Cloud/Local VLM Reasoning companion
        if (enableVlm && p.current_file) {
          try {
            // Find media item and get preview as base64
            const item = get(mediaItems).find(i => i.file_name === p.current_file);
            if (item) {
              const previewPath = item.preview_path || item.file_path;
              const base64Data = await invoke<string>('read_image_base64', { path: previewPath });
              const base64Clean = base64Data.split(',')[1];
              
              let responseText = '';
              let promptText = 'Describe this photo composition and quality in 10 words or less.';

              if (enableStructuredCulling) {
                promptText = `Analyze this photo's composition, lighting, focus, and overall quality. Return EXACTLY a JSON object matching this schema:
{
  "category": "best" | "trash" | "draft" | "review" | "none",
  "rating": 0 | 1 | 2 | 3 | 4 | 5,
  "colorLabel": "red" | "orange" | "yellow" | "green" | "none",
  "reason": "short explanation"
}`;
              }
              
              if (vlmProvider === 'ollama') {
                addLog(`[VLM] Generating Ollama recommendation for ${p.current_file}...`, 'info');
                responseText = await invoke<string>('query_ollama_vision', {
                  imageBase64: base64Clean,
                  model: ollamaModel,
                  prompt: promptText,
                  ollamaUrl: ollamaUrl
                });
              } else if (vlmProvider === 'nvidia_nim') {
                if (!nvidiaApiKey) {
                  addLog(`[VLM WARN] NVIDIA NIM API key is missing. Skipping description.`, 'warn');
                } else {
                  addLog(`[VLM] Generating NVIDIA NIM recommendation for ${p.current_file}...`, 'info');
                  responseText = await invoke<string>('query_nvidia_nim_vision', {
                    imageBase64: base64Clean,
                    model: nvidiaModel,
                    prompt: promptText,
                    apiKey: nvidiaApiKey
                  });
                }
              } else if (vlmProvider === 'gemini') {
                if (!geminiApiKey) {
                  addLog(`[VLM WARN] Gemini API key is missing. Skipping recommendation.`, 'warn');
                } else {
                  addLog(`[VLM] Generating Google Gemini recommendation for ${p.current_file}...`, 'info');
                  responseText = await invoke<string>('query_gemini_vision', {
                    imageBase64: base64Clean,
                    model: geminiModel,
                    prompt: promptText,
                    apiKey: geminiApiKey
                  });
                }
              }

              if (responseText) {
                if (enableStructuredCulling) {
                  try {
                    const resultJson = parseJsonFromText(responseText);
                    const reason = resultJson.reason || '';
                    const category = resultJson.category || 'none';
                    const rating = typeof resultJson.rating === 'number' ? resultJson.rating : 0;
                    const colorLabel = resultJson.colorLabel || 'none';

                    addLog(`[VLM DECISION] ${p.current_file}: Category=${category.toUpperCase()} | Rating=${rating}* | Label=${colorLabel} | Reason="${reason}"`, 'success');

                    // Map VLM category to DB category ID
                    let categoryId: number | null = null;
                    let vlmCat: 'best' | 'trash' | 'review' = 'review';
                    if (category === 'best') {
                      categoryId = 2;
                      vlmCat = 'best';
                    } else if (category === 'trash') {
                      categoryId = 1;
                      vlmCat = 'trash';
                    } else if (category === 'draft') {
                      categoryId = 3;
                      vlmCat = 'review';
                    } else if (category === 'review') {
                      categoryId = 4;
                      vlmCat = 'review';
                    }

                    // Save VLM decisions back to database using store actions
                    await setCategoryForItem(item.id, proj.id, categoryId);
                    if (rating > 0) {
                      await setStarRatingForItem(item.id, rating);
                    }
                    if (colorLabel && colorLabel !== 'none') {
                      await setColorLabelForItem(item.id, colorLabel);
                    }

                    // Update UI stats to reflect VLM choice instead of initial choice
                    if (initialCat !== vlmCat) {
                      stats[initialCat]--;
                      stats[vlmCat]++;
                    }
                  } catch (jsonErr: any) {
                    addLog(`[VLM ERROR] Failed to parse JSON recommendation: ${jsonErr.message}. Output was: "${responseText}"`, 'error');
                  }
                } else {
                  addLog(`[VLM DESC] ${p.current_file}: "${responseText.trim()}"`, 'success');
                }
              }
            }
          } catch (e: any) {
            addLog(`[VLM WARN] VLM companion failed: ${e.message || e}`, 'warn');
          }
        }
      } else if (p.phase === 'duplicates') {
        addLog(`[SYSTEM] ${p.message}`, 'warn');
      } else if (p.phase === 'complete') {
        addLog(`[SUCCESS] ${p.message}`, 'success');
        isCulling = false;
        unlistenAi();
        toast.success('Local AI Culling Completed!');
        
        // Reload project category stats & Svelte view grid
        await loadCategoryStats(proj.id);
        // Navigate to first item to trigger cache reload
        navigateTo(0);
      }
    });

    try {
      await invoke('run_local_ai_cull', {
        projectId: proj.id,
        aestheticWeight,
        technicalWeight,
        faceWeight,
        duplicateThreshold
      });
    } catch (err: any) {
      isCulling = false;
      unlistenAi();
      addLog(`[ERROR] Execution aborted: ${err.message}`, 'error');
      toast.error(`Culling execution failed: ${err.message}`);
    }
  }

  async function runLearnMyStyle() {
    const proj = get(currentProject);
    if (!proj) return;
    
    if (styleDatasetSize < 5) {
      toast.error('You need at least 5 manual culls (Best/Trash) to train style models!');
      return;
    }

    addLog('Training lightweight KNN classification model on CLIP embeddings...', 'info');
    try {
      const predictedCount = await invoke<number>('train_and_predict_style', {
        projectId: proj.id
      });
      addLog(`[LEARN MY STYLE] Trained & predicted style for ${predictedCount} uncategorized photos!`, 'success');
      toast.success(`Personalized culling complete: predicted categories for ${predictedCount} photos.`);
      
      // Reload stats
      await loadCategoryStats(proj.id);
      navigateTo(0);
    } catch (err: any) {
      addLog(`[ERROR] Training failed: ${err.message}`, 'error');
      toast.error(`Training failed: ${err.message}`);
    }
  }

  function handleClose() {
    if (isCulling) {
      const confirmClose = confirm('AI culling is actively processing. Stop and close?');
      if (confirmClose) {
        showAiCull.set(false);
      }
    } else {
      showAiCull.set(false);
    }
  }

  function exportCsvReport() {
    if (consoleLogs.length === 0) return;
    
    let csvContent = 'data:text/csv;charset=utf-8,Type,Log Message\n';
    consoleLogs.forEach(log => {
      csvContent += `"${log.type}","${log.text.replace(/"/g, '""')}"\n`;
    });
    
    const encodedUri = encodeURI(csvContent);
    const link = document.createElement('a');
    link.setAttribute('href', encodedUri);
    link.setAttribute('download', `keepix_local_ai_cull_report.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }
</script>

{#if $showAiCull}
  <div class="modal-overlay" role="presentation" onclick={handleClose}>
    <div class="modal-content glass-card ai-cull-modal" onclick={(e) => e.stopPropagation()} role="dialog">
      <!-- Header -->
      <div class="modal-header">
        <div class="header-title">
          <span class="pulse-dot"></span>
          <h3>Local Neural Culling Workstation v4.1.0</h3>
        </div>
        <button class="btn-close" onclick={handleClose} title="Close">
          <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="4" y1="4" x2="14" y2="14"/>
            <line x1="14" y1="4" x2="4" y2="14"/>
          </svg>
        </button>
      </div>

      <!-- Main Panel -->
      <div class="modal-body">
        {#if !hasModels}
          <!-- SETUP DOWNLOAD WIZARD -->
          <div class="setup-wizard">
            <div class="setup-icon">🧠</div>
            <h4>Initialize Local Neural Workstation</h4>
            <p class="setup-desc">
              Keepix performs 100% private, accelerated AI scoring on your hardware. We need to download the standard ONNX weights into your AppData directory first.
            </p>

            <div class="model-list">
              <div class="model-item">
                <div class="model-info">
                  <span>CLIP Image Vectorizer (ViT-B/32)</span>
                  <span class="size-badge">150 MB</span>
                </div>
                <div class="progress-bar">
                  <div class="progress-fill" style="width: {downloadProgress['clip_image.onnx']}%"></div>
                </div>
              </div>

              <div class="model-item">
                <div class="model-info">
                  <span>LAION Aesthetics Scorer</span>
                  <span class="size-badge">5 MB</span>
                </div>
                <div class="progress-bar">
                  <div class="progress-fill" style="width: {downloadProgress['laion_aesthetic.onnx']}%"></div>
                </div>
              </div>

              <div class="model-item">
                <div class="model-info">
                  <span>MUSIQ Technical Quality Scorer</span>
                  <span class="size-badge">90 MB</span>
                </div>
                <div class="progress-bar">
                  <div class="progress-fill" style="width: {downloadProgress['technical_musiq.onnx']}%"></div>
                </div>
              </div>

              <div class="model-item">
                <div class="model-info">
                  <span>UltraFace RFB Detector</span>
                  <span class="size-badge">1.5 MB</span>
                </div>
                <div class="progress-bar">
                  <div class="progress-fill" style="width: {downloadProgress['face_detector.onnx']}%"></div>
                </div>
              </div>
            </div>

            <div class="setup-footer">
              <span class="status-msg">{downloadStatusText}</span>
              <button class="btn btn-glowing" disabled={isDownloading} onclick={startDownloadingModels}>
                {#if isDownloading}
                  Downloading Models...
                {:else}
                  Download & Initialize (246.5MB)
                {/if}
              </button>
            </div>
          </div>
        {:else}
          <!-- WORKSTATION DASHBOARD -->
          <div class="dashboard-grid">
            <!-- Tuning Panel (Left) -->
            <div class="dashboard-col tuning-panel glass-inner">
              <h5>Neural Weights</h5>
              <div class="control-group">
                <label for="aesthetic-weight">Aesthetic Quality Weight: {(aestheticWeight * 100).toFixed(0)}%</label>
                <input type="range" id="aesthetic-weight" min="0" max="1" step="0.05" bind:value={aestheticWeight} />
              </div>

              <div class="control-group">
                <label for="technical-weight">Technical Sharpness Weight: {(technicalWeight * 100).toFixed(0)}%</label>
                <input type="range" id="technical-weight" min="0" max="1" step="0.05" bind:value={technicalWeight} />
              </div>

              <div class="control-group">
                <label for="face-weight">Eye & Face Penalty Weight: {(faceWeight * 100).toFixed(0)}%</label>
                <input type="range" id="face-weight" min="0" max="1" step="0.05" bind:value={faceWeight} />
              </div>

              <div class="control-group">
                <label for="dup-threshold">Duplicate Sensitivity Threshold: {(duplicateThreshold * 100).toFixed(0)}%</label>
                <input type="range" id="dup-threshold" min="0.80" max="0.99" step="0.01" bind:value={duplicateThreshold} />
              </div>

              <hr />

              <h5>"Learn My Style" Personalized Culling</h5>
              <div class="knn-stats">
                <span class="stat-lbl">Manual Labeled Samples:</span>
                <span class="stat-val">{styleDatasetSize} culls</span>
              </div>
              <div class="control-group">
                <label for="knn-confidence">KNN Confidence Threshold: {(knnConfidenceThreshold * 100).toFixed(0)}%</label>
                <input type="range" id="knn-confidence" min="0.50" max="0.95" step="0.05" bind:value={knnConfidenceThreshold} />
              </div>
              <button class="btn btn-style" disabled={styleDatasetSize < 5 || isCulling} onclick={runLearnMyStyle}>
                Personalize & Cull (KNN)
              </button>

              <hr />

              <h5>VLM Reasoning Companion</h5>
              <div class="ollama-toggle">
                <label for="enable-vlm">Run Parallel VLM Reasoning</label>
                <input type="checkbox" id="enable-vlm" bind:checked={enableVlm} />
              </div>
              {#if enableVlm}
                <div class="ollama-toggle">
                  <label for="enable-structured-culling">Use VLM for Auto-Culling</label>
                  <input type="checkbox" id="enable-structured-culling" bind:checked={enableStructuredCulling} />
                </div>
                <div class="control-group">
                  <label for="vlm-provider">VLM Provider</label>
                  <select id="vlm-provider" bind:value={vlmProvider} class="cyber-select">
                    <option value="ollama">Local Ollama</option>
                    <option value="nvidia_nim">NVIDIA NIM (Cloud API)</option>
                    <option value="gemini">Google Gemini (Cloud API)</option>
                  </select>
                </div>
                
                {#if vlmProvider === 'ollama'}
                  <div class="control-group">
                    <label for="ollama-url">Ollama Node URL</label>
                    <input type="text" id="ollama-url" bind:value={ollamaUrl} />
                  </div>
                  <div class="control-group">
                    <label for="ollama-model">Vision Model</label>
                    <input type="text" id="ollama-model" bind:value={ollamaModel} />
                  </div>
                  <button class="btn btn-outline-cyber" onclick={testOllamaConnection}>
                    Test Connection ({ollamaStatus})
                  </button>
                {:else if vlmProvider === 'nvidia_nim'}
                  <div class="control-group">
                    <label for="nvidia-api-key">NVIDIA API Key</label>
                    <input type="password" id="nvidia-api-key" placeholder="nvapi-..." bind:value={nvidiaApiKey} />
                  </div>
                  <div class="control-group">
                    <label for="nvidia-model">NIM Vision Model</label>
                    <select id="nvidia-model" bind:value={nvidiaModel} class="cyber-select">
                      <option value="moonshotai/kimi-k2.6">moonshotai/kimi-k2.6 (Recommended)</option>
                      <option value="meta/llama-3.2-11b-vision-instruct">meta/llama-3.2-11b-vision-instruct</option>
                      <option value="nvidia/vila-15b">nvidia/vila-15b (Premium Visuals)</option>
                      <option value="microsoft/phi-3.5-vision-instruct">microsoft/phi-3.5-vision-instruct (Fast & Sharp)</option>
                    </select>
                  </div>
                  <button class="btn btn-outline-cyber" onclick={testNvidiaConnection}>
                    Test Connection ({nvidiaStatus})
                  </button>
                {:else if vlmProvider === 'gemini'}
                  <div class="control-group">
                    <label for="gemini-api-key">Gemini API Key</label>
                    <input type="password" id="gemini-api-key" placeholder="AIzaSy..." bind:value={geminiApiKey} />
                  </div>
                  <div class="control-group">
                    <label for="gemini-model">Gemini Model</label>
                    <select id="gemini-model" bind:value={geminiModel} class="cyber-select">
                      <option value="gemini-2.5-flash">gemini-2.5-flash (Fast & Recommended)</option>
                      <option value="gemini-2.5-pro">gemini-2.5-pro (High Quality)</option>
                      <option value="gemini-1.5-flash">gemini-1.5-flash (Legacy Fast)</option>
                      <option value="gemini-1.5-pro">gemini-1.5-pro (Legacy High Quality)</option>
                    </select>
                  </div>
                  <button class="btn btn-outline-cyber" onclick={testGeminiConnection}>
                    Test Connection ({geminiStatus})
                  </button>
                {/if}
              {/if}
            </div>

            <!-- Viewport Panel (Center) -->
            <div class="dashboard-col viewport-panel glass-inner">
              {#if isCulling && currentFile}
                <div class="analysis-card">
                  <span class="analysis-title">Analyzing: {currentFile}</span>
                  
                  <!-- Quality Gauges -->
                  <div class="gauges-container">
                    <div class="gauge-item">
                      <div class="circular-progress">
                        <span class="gauge-value">{currentAesthetic.toFixed(1)}</span>
                        <svg>
                          <circle cx="40" cy="40" r="35" style="stroke-dasharray: 220; stroke-dashoffset: {220 - (currentAesthetic / 10) * 220}"></circle>
                        </svg>
                      </div>
                      <span class="gauge-label">Aesthetic</span>
                    </div>

                    <div class="gauge-item">
                      <div class="circular-progress">
                        <span class="gauge-value">{currentTechnical.toFixed(0)}</span>
                        <svg>
                          <circle cx="40" cy="40" r="35" style="stroke-dasharray: 220; stroke-dashoffset: {220 - (currentTechnical / 100) * 220}"></circle>
                        </svg>
                      </div>
                      <span class="gauge-label">Technical</span>
                    </div>

                    <div class="gauge-item">
                      <div class="circular-progress neon-overall">
                        <span class="gauge-value">{(currentOverall * 100).toFixed(0)}%</span>
                        <svg>
                          <circle cx="40" cy="40" r="35" style="stroke-dasharray: 220; stroke-dashoffset: {220 - currentOverall * 220}"></circle>
                        </svg>
                      </div>
                      <span class="gauge-label">Overall Match</span>
                    </div>
                  </div>

                  <!-- Face alerts -->
                  <div class="face-alerts">
                    <div class="alert-badge {currentFaceCount > 0 ? 'badge-green' : 'badge-gray'}">
                      Faces Detected: {currentFaceCount}
                    </div>
                    {#if currentBlinkDetected}
                      <div class="alert-badge badge-red flash">
                        ⚠️ Eyes Closed Detected
                      </div>
                    {/if}
                  </div>

                  <div class="action-recommendation">
                    {#if currentOverall > 0.75}
                      <span class="rec-badge badge-keep">KEEP (BEST RECOMMENDATION)</span>
                    {:else if currentOverall < 0.25}
                      <span class="rec-badge badge-discard">DISCARD (TRASH RECOMMENDATION)</span>
                    {:else}
                      <span class="rec-badge badge-review">AMBIGUOUS (HUMAN REVIEW RECOMMENDED)</span>
                    {/if}
                  </div>
                </div>
              {:else}
                <div class="idle-viewport">
                  <div class="terminal-cube">🧊</div>
                  <p>Neural Workstation Idle</p>
                  <button class="btn btn-glowing btn-large" onclick={executeLocalCulling}>
                    Launch Parallel AI Cull Scan
                  </button>
                </div>
              {/if}
            </div>

            <!-- Operations Console (Right) -->
            <div class="dashboard-col console-panel glass-inner">
              <div class="console-header">
                <h5>Operations Log</h5>
                <button class="btn-export" title="Export CSV" onclick={exportCsvReport}>
                  📥 Export
                </button>
              </div>

              <!-- Terminal view -->
              <div class="terminal-box" bind:this={logsContainer}>
                {#if consoleLogs.length === 0}
                  <span class="term-placeholder">System ready. Awaiting neural culling sequence...</span>
                {/if}
                {#each consoleLogs as log}
                  <div class="log-line log-{log.type}">
                    {log.text}
                  </div>
                {/each}
              </div>

              <!-- Mini Stats -->
              <div class="console-stats">
                <div class="c-stat-box">
                  <span class="c-stat-val val-best">{stats.best}</span>
                  <span class="c-stat-lbl">Keep</span>
                </div>
                <div class="c-stat-box">
                  <span class="c-stat-val val-trash">{stats.trash}</span>
                  <span class="c-stat-lbl">Trash</span>
                </div>
                <div class="c-stat-box">
                  <span class="c-stat-val val-review">{stats.review}</span>
                  <span class="c-stat-lbl">Review</span>
                </div>
                <div class="c-stat-box">
                  <span class="c-stat-val val-progress">{processedPhotos}/{totalPhotos}</span>
                  <span class="c-stat-lbl">Progress</span>
                </div>
              </div>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  /* Cyberpunk Glassmorphic styling */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(8, 8, 16, 0.75);
    backdrop-filter: blur(8px);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }

  .modal-content.ai-cull-modal {
    width: 1200px;
    height: 750px;
    background: rgba(14, 15, 27, 0.85);
    border: 1px solid rgba(0, 242, 254, 0.25);
    box-shadow: 0 0 30px rgba(0, 242, 254, 0.15), inset 0 0 20px rgba(255, 255, 255, 0.02);
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    color: #e2e8f0;
    font-family: 'Inter', sans-serif;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 18px 24px;
    background: rgba(20, 21, 38, 0.5);
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .pulse-dot {
    width: 8px;
    height: 8px;
    background: #00f2fe;
    border-radius: 50%;
    box-shadow: 0 0 10px #00f2fe;
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(0, 242, 254, 0.7); }
    70% { transform: scale(1); box-shadow: 0 0 0 8px rgba(0, 242, 254, 0); }
    100% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(0, 242, 254, 0); }
  }

  .modal-header h3 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    letter-spacing: 0.5px;
    color: #f8fafc;
  }

  .btn-close {
    background: none;
    border: none;
    color: #94a3b8;
    cursor: pointer;
    transition: color 0.2s;
  }

  .btn-close:hover {
    color: #ef4444;
  }

  .modal-body {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  /* SETUP WIZARD */
  .setup-wizard {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px;
  }

  .setup-icon {
    font-size: 4rem;
    margin-bottom: 16px;
    filter: drop-shadow(0 0 20px rgba(0, 242, 254, 0.3));
  }

  .setup-wizard h4 {
    margin: 0 0 10px 0;
    font-size: 1.5rem;
    font-weight: 700;
    color: #f8fafc;
  }

  .setup-desc {
    max-width: 600px;
    text-align: center;
    color: #94a3b8;
    line-height: 1.6;
    margin-bottom: 30px;
  }

  .model-list {
    width: 500px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    margin-bottom: 35px;
  }

  .model-item {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    padding: 12px 16px;
  }

  .model-info {
    display: flex;
    justify-content: space-between;
    margin-bottom: 8px;
    font-size: 0.9rem;
    font-weight: 500;
  }

  .size-badge {
    background: rgba(0, 242, 254, 0.1);
    color: #00f2fe;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .progress-bar {
    width: 100%;
    height: 4px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #00f2fe, #4facfe);
    box-shadow: 0 0 8px #00f2fe;
    transition: width 0.1s ease;
  }

  .setup-footer {
    width: 500px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .status-msg {
    font-size: 0.85rem;
    color: #94a3b8;
  }

  /* BUTTONS */
  .btn {
    border: none;
    border-radius: 6px;
    padding: 10px 20px;
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-glowing {
    background: linear-gradient(90deg, #00f2fe, #4facfe);
    color: #0f172a;
    box-shadow: 0 0 15px rgba(0, 242, 254, 0.4);
  }

  .btn-glowing:hover:not(:disabled) {
    box-shadow: 0 0 25px rgba(0, 242, 254, 0.6);
    transform: translateY(-1px);
  }

  .btn-glowing:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* DASHBOARD GRID */
  .dashboard-grid {
    display: grid;
    grid-template-columns: 280px 1fr 340px;
    width: 100%;
    height: 100%;
    padding: 16px;
    gap: 16px;
    box-sizing: border-box;
  }

  .dashboard-col {
    display: flex;
    flex-direction: column;
    border-radius: 8px;
    overflow: hidden;
  }

  .glass-inner {
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid rgba(255, 255, 255, 0.04);
    box-shadow: inset 0 0 10px rgba(255, 255, 255, 0.01);
    padding: 16px;
  }

  /* TUNING PANEL */
  .tuning-panel {
    overflow-y: auto;
  }

  .tuning-panel h5 {
    margin: 0 0 16px 0;
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: #94a3b8;
    border-left: 2px solid #00f2fe;
    padding-left: 8px;
  }

  .control-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 20px;
  }

  .control-group label {
    font-size: 0.8rem;
    font-weight: 500;
    color: #cbd5e1;
  }

  .control-group input[type="range"] {
    appearance: none;
    -webkit-appearance: none;
    height: 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    outline: none;
  }

  .control-group input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #00f2fe;
    box-shadow: 0 0 6px #00f2fe;
    cursor: pointer;
    transition: transform 0.1s;
  }

  .control-group input[type="range"]::-webkit-slider-thumb:hover {
    transform: scale(1.2);
  }

  .control-group input[type="text"],
  .control-group input[type="password"],
  .cyber-select {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 4px;
    padding: 6px 10px;
    color: #e2e8f0;
    font-size: 0.85rem;
    outline: none;
    transition: border-color 0.2s;
  }

  .cyber-select option {
    background: #141526;
    color: #e2e8f0;
  }

  .control-group input[type="text"]:focus,
  .control-group input[type="password"]:focus,
  .cyber-select:focus {
    border-color: #00f2fe;
    box-shadow: 0 0 8px rgba(0, 242, 254, 0.2);
  }

  hr {
    border: 0;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    margin: 20px 0;
  }

  .knn-stats {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
    margin-bottom: 12px;
  }

  .btn-style {
    width: 100%;
    background: linear-gradient(90deg, #d946ef, #a855f7);
    color: #ffffff;
    box-shadow: 0 0 15px rgba(217, 70, 239, 0.3);
  }

  .btn-style:hover:not(:disabled) {
    box-shadow: 0 0 25px rgba(217, 70, 239, 0.5);
    transform: translateY(-1px);
  }

  .btn-style:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .ollama-toggle {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.85rem;
    margin-bottom: 16px;
  }

  .btn-outline-cyber {
    background: none;
    border: 1px solid rgba(0, 242, 254, 0.3);
    color: #00f2fe;
    font-size: 0.8rem;
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-outline-cyber:hover {
    background: rgba(0, 242, 254, 0.05);
    border-color: #00f2fe;
  }

  /* VIEWPORT PANEL */
  .viewport-panel {
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .idle-viewport {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
  }

  .terminal-cube {
    font-size: 3.5rem;
    animation: float 4s ease-in-out infinite;
  }

  @keyframes float {
    0% { transform: translateY(0px) rotate(0deg); }
    50% { transform: translateY(-10px) rotate(15deg); }
    100% { transform: translateY(0px) rotate(0deg); }
  }

  .idle-viewport p {
    color: #94a3b8;
    font-size: 0.95rem;
    margin: 0;
  }

  .btn-large {
    padding: 14px 28px;
    font-size: 1rem;
  }

  .analysis-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    max-width: 500px;
    background: rgba(255, 255, 255, 0.01);
    border: 1px solid rgba(255, 255, 255, 0.03);
    border-radius: 12px;
    padding: 24px;
    box-sizing: border-box;
  }

  .analysis-title {
    font-size: 0.9rem;
    font-weight: 600;
    color: #f8fafc;
    margin-bottom: 24px;
    word-break: break-all;
    text-align: center;
  }

  .gauges-container {
    display: flex;
    justify-content: space-around;
    width: 100%;
    margin-bottom: 24px;
  }

  .gauge-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }

  .circular-progress {
    position: relative;
    width: 80px;
    height: 80px;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .gauge-value {
    font-size: 0.95rem;
    font-weight: 700;
    color: #f8fafc;
  }

  .circular-progress svg {
    position: absolute;
    width: 80px;
    height: 80px;
    transform: rotate(-90deg);
  }

  .circular-progress circle {
    cx: 40;
    cy: 40;
    r: 35;
    fill: none;
    stroke: #00f2fe;
    stroke-width: 4px;
    stroke-linecap: round;
    filter: drop-shadow(0 0 5px #00f2fe);
    transition: stroke-dashoffset 0.3s ease;
  }

  .circular-progress.neon-overall circle {
    stroke: #d946ef;
    filter: drop-shadow(0 0 5px #d946ef);
  }

  .gauge-label {
    font-size: 0.75rem;
    color: #94a3b8;
    font-weight: 500;
  }

  .face-alerts {
    display: flex;
    gap: 12px;
    margin-bottom: 24px;
  }

  .alert-badge {
    padding: 6px 12px;
    border-radius: 20px;
    font-size: 0.75rem;
    font-weight: 600;
    border: 1px solid transparent;
  }

  .badge-green {
    background: rgba(34, 197, 94, 0.1);
    color: #22c55e;
    border-color: rgba(34, 197, 94, 0.2);
  }

  .badge-gray {
    background: rgba(255, 255, 255, 0.02);
    color: #94a3b8;
    border-color: rgba(255, 255, 255, 0.05);
  }

  .badge-red {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
    border-color: rgba(239, 68, 68, 0.3);
    box-shadow: 0 0 10px rgba(239, 68, 68, 0.2);
  }

  .flash {
    animation: flash-anim 1.5s infinite;
  }

  @keyframes flash-anim {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .action-recommendation {
    width: 100%;
    display: flex;
    justify-content: center;
  }

  .rec-badge {
    padding: 10px 20px;
    border-radius: 6px;
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: 0.5px;
    width: 100%;
    text-align: center;
    border: 1px solid transparent;
  }

  .badge-keep {
    background: rgba(34, 197, 94, 0.1);
    color: #22c55e;
    border-color: rgba(34, 197, 94, 0.3);
    box-shadow: 0 0 15px rgba(34, 197, 94, 0.15);
  }

  .badge-discard {
    background: rgba(239, 68, 68, 0.1);
    color: #ef4444;
    border-color: rgba(239, 68, 68, 0.3);
    box-shadow: 0 0 15px rgba(239, 68, 68, 0.15);
  }

  .badge-review {
    background: rgba(245, 158, 11, 0.1);
    color: #f59e0b;
    border-color: rgba(245, 158, 11, 0.3);
  }

  /* CONSOLE PANEL */
  .console-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .console-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .console-header h5 {
    margin: 0;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #94a3b8;
  }

  .btn-export {
    background: none;
    border: none;
    color: #94a3b8;
    cursor: pointer;
    font-size: 0.8rem;
    font-weight: 600;
    transition: color 0.2s;
  }

  .btn-export:hover {
    color: #00f2fe;
  }

  .terminal-box {
    flex: 1;
    background: rgba(8, 8, 16, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.03);
    border-radius: 6px;
    padding: 12px;
    font-family: 'Consolas', 'Courier New', monospace;
    font-size: 0.75rem;
    line-height: 1.5;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 16px;
  }

  .term-placeholder {
    color: #475569;
  }

  .log-line {
    word-break: break-all;
  }

  .log-info { color: #94a3b8; }
  .log-success { color: #22c55e; }
  .log-warn { color: #f59e0b; }
  .log-error { color: #ef4444; }

  .console-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
  }

  .c-stat-box {
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid rgba(255, 255, 255, 0.04);
    border-radius: 6px;
    padding: 8px;
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .c-stat-val {
    font-size: 0.95rem;
    font-weight: 700;
  }

  .val-best { color: #22c55e; }
  .val-trash { color: #ef4444; }
  .val-review { color: #f59e0b; }
  .val-progress { color: #00f2fe; }

  .c-stat-lbl {
    font-size: 0.65rem;
    color: #94a3b8;
    margin-top: 4px;
    text-transform: uppercase;
    font-weight: 500;
  }
</style>
