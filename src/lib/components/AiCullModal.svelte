<script lang="ts">
  import { showAiCull } from '$lib/stores/ui';
  import { mediaItems, displayItems, setCategoryForItem, setStarRatingForItem, setColorLabelForItem, loadCategoryStats, currentItem, selectedIndex, navigateTo } from '$lib/stores/media';
  import { get } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';
  import { toast } from '$lib/stores/toast';
  import { t, locale } from '$lib/i18n';
  import { onMount } from 'svelte';
  import { getCategoryColor } from '$lib/types';
  import { DEFAULT_ADJUSTMENTS } from '$lib/services/image-editor';

  // Custom Templates Definition
  interface CustomTemplate {
    id: string;
    name: string;
    prompt: string;
    aesthetic: string;
  }
  let customTemplates = $state<CustomTemplate[]>([]);

  // Prompt Presets Definitions
  const PROMPT_PRESETS = {
    technical: {
      en: 'Keep the sharpest, most in-focus photos as BEST. Mark blurry, duplicate test shots, or extremely poorly exposed frames as TRASH. Mark minor lighting or minor framing issues as REVIEW.',
      id: 'Pilih foto terbaik yang fokus tajam dan detail. Buang foto yang blur, buram, duplikat tes, atau salah pencahayaan parah ke TRASH. Tandai foto dengan sedikit isu pencahayaan atau komposisi untuk direview.'
    },
    portrait: {
      en: 'Keep sharp, expressive portraits with open eyes and natural smiles as BEST. Mark out-of-focus portraits, closed eyes/blinking, or duplicate series as TRASH. Mark candidate alternates as DRAFT, and ambiguous expressions as REVIEW.',
      id: 'Simpan potret tajam dan ekspresif dengan mata terbuka dan senyum natural sebagai BEST. Buang potret buram, mata tertutup/berkedip, atau duplikat ke TRASH. Tandai alternatif kedua sebagai DRAFT, ekspresi ambigu sebagai REVIEW.'
    },
    landscape: {
      en: 'Keep landscape photos with strong composition, dramatic lighting, and sharp details as BEST. Trash overexposed/underexposed landscape tests or duplicate frames. Mark interesting shots with minor lighting issues as REVIEW.',
      id: 'Simpan foto lanskap dengan komposisi kuat, pencahayaan dramatis, dan detail tajam sebagai BEST. Buang duplikat test shot atau foto over/underexposed parah ke TRASH. Tandai foto menarik dengan isu minor sebagai REVIEW.'
    },
    event: {
      en: 'Keep key moments, emotional candid shots, and clean group photos as BEST. Trash duplicate test shots, out-of-focus action, or blinking eyes. Mark secondary backups as DRAFT, and ambiguous candids as REVIEW.',
      id: 'Simpan momen penting, candid penuh emosi, dan foto grup yang bersih sebagai BEST. Buang duplikat tes, aksi yang blur, atau mata berkedip ke TRASH. Tandai cadangan kedua sebagai DRAFT, ekspresi ambigu sebagai REVIEW.'
    },
    custom: {
      en: '',
      id: ''
    }
  };

  const AESTHETIC_STYLES = {
    none: { en: 'AI Decides (Free form)', id: 'Ditentukan AI (Bebas)' },
    natural: { en: 'Natural Correction (Standard enhancement)', id: 'Koreksi Alami (Koreksi standar)' },
    cinematic: { en: 'Moody Cinematic (Rich colors, deeper shadows, cool tints)', id: 'Sinematik Dramatis (Warna pekat, bayangan dalam)' },
    vintage: { en: 'Warm Vintage (Warm temperature, lower contrast, soft highlights)', id: 'Klasik Hangat (Suhu hangat, kontras rendah)' },
    bw: { en: 'Classic Black & White (High contrast monochrome)', id: 'Hitam Putih Klasik (Monokrom kontras tinggi)' },
    punchy: { en: 'Vibrant Punchy (Slight exposure boost, vivid saturation)', id: 'Vibrant & Cerah (Eksposur naik, saturasi tinggi)' }
  };

  // Config State (saved in localStorage)
  let apiKey = $state('');
  let modelName = $state('gemini-2.5-flash');
  let customPrompt = $state('');
  let cullScope = $state('uncategorized'); // 'uncategorized' | 'all'
  let enableAutoEdit = $state(false);
  let selectedPreset = $state('technical');
  let targetAesthetic = $state('natural');
  let logFilter = $state('all'); // 'all' | 'best' | 'trash' | 'warning'

  // Run State
  let isCulling = $state(false);
  let isPaused = $state(false);
  let totalToCull = $state(0);
  let currentCullIndex = $state(0);
  let currentItemName = $state('');
  let currentItemSrc = $state('');

  // Stats
  let stats = $state({
    best: 0,
    trash: 0,
    draft: 0,
    review: 0,
    skipped: 0
  });

  // ETR & Performance tracking
  let startTime = $state<number | null>(null);
  let elapsedTime = $state(0);
  let speed = $state(0); // seconds per photo
  let etr = $state('--');

  // Console Logs
  interface CullLog {
    timestamp: string;
    fileName: string;
    status: 'success' | 'warning' | 'error' | 'info';
    message: string;
  }
  let logs = $state<CullLog[]>([]);
  let consoleContainer = $state<HTMLDivElement>();

  // Derived logs filter
  let filteredLogs = $derived.by(() => {
    if (logFilter === 'all') return logs;
    if (logFilter === 'best') return logs.filter(l => l.status === 'success');
    if (logFilter === 'trash') return logs.filter(l => l.status === 'error' && !l.message.includes('Failed:'));
    if (logFilter === 'warning') return logs.filter(l => l.status === 'warning' || l.status === 'error' || l.message.includes('Failed:'));
    return logs;
  });

  // Derived composition stats
  let totalCount = $derived(stats.best + stats.draft + stats.review + stats.trash + stats.skipped);
  let bestPct = $derived(totalCount > 0 ? (stats.best / totalCount) * 100 : 0);
  let draftPct = $derived(totalCount > 0 ? (stats.draft / totalCount) * 100 : 0);
  let reviewPct = $derived(totalCount > 0 ? (stats.review / totalCount) * 100 : 0);
  let trashPct = $derived(totalCount > 0 ? (stats.trash / totalCount) * 100 : 0);
  let skippedPct = $derived(totalCount > 0 ? (stats.skipped / totalCount) * 100 : 0);

  // Load configuration from local storage
  onMount(() => {
    if (typeof localStorage !== 'undefined') {
      apiKey = localStorage.getItem('keepix_gemini_api_key') || '';
      modelName = localStorage.getItem('keepix_gemini_model') || 'gemini-2.5-flash';
      enableAutoEdit = localStorage.getItem('keepix_gemini_auto_edit') === 'true';
      selectedPreset = localStorage.getItem('keepix_gemini_cull_preset') || 'technical';
      targetAesthetic = localStorage.getItem('keepix_gemini_aesthetic') || 'natural';
      
      const loadedTemplates = localStorage.getItem('keepix_custom_cull_templates');
      if (loadedTemplates) {
        try {
          customTemplates = JSON.parse(loadedTemplates);
        } catch (e) {
          console.error('Failed to parse custom templates', e);
        }
      }

      const loadedRules = localStorage.getItem('keepix_gemini_cull_rules');
      if (loadedRules) {
        customPrompt = loadedRules;
      } else {
        const localeKey = $locale === 'id' ? 'id' : 'en';
        customPrompt = PROMPT_PRESETS[selectedPreset as keyof typeof PROMPT_PRESETS]?.[localeKey] || '';
      }
    }
  });

  // Automatically update prompt when preset selection changes
  $effect(() => {
    if (selectedPreset !== 'custom' && !selectedPreset.startsWith('custom-')) {
      const localeKey = $locale === 'id' ? 'id' : 'en';
      customPrompt = PROMPT_PRESETS[selectedPreset as keyof typeof PROMPT_PRESETS][localeKey];
    } else if (selectedPreset.startsWith('custom-')) {
      const found = customTemplates.find(t => t.id === selectedPreset);
      if (found) {
        customPrompt = found.prompt;
        targetAesthetic = found.aesthetic || 'natural';
      }
    }
  });

  function saveConfig() {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('keepix_gemini_api_key', apiKey);
      localStorage.setItem('keepix_gemini_model', modelName);
      localStorage.setItem('keepix_gemini_cull_rules', customPrompt);
      localStorage.setItem('keepix_gemini_auto_edit', enableAutoEdit.toString());
      localStorage.setItem('keepix_gemini_cull_preset', selectedPreset);
      localStorage.setItem('keepix_gemini_aesthetic', targetAesthetic);
      saveCustomTemplates();
    }
  }

  function saveCustomTemplates() {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('keepix_custom_cull_templates', JSON.stringify(customTemplates));
    }
  }

  function handleCreateTemplate() {
    const name = prompt($locale === 'id' ? 'Masukkan nama template baru:' : 'Enter new template name:');
    if (!name || !name.trim()) return;
    
    const newId = `custom-${Date.now()}`;
    const newTmpl: CustomTemplate = {
      id: newId,
      name: name.trim(),
      prompt: customPrompt,
      aesthetic: targetAesthetic
    };
    
    customTemplates = [...customTemplates, newTmpl];
    saveCustomTemplates();
    selectedPreset = newId;
    toast.success($locale === 'id' ? 'Template dibuat!' : 'Template created!');
  }

  function handleSaveTemplate() {
    if (!selectedPreset.startsWith('custom-')) return;
    customTemplates = customTemplates.map(t => {
      if (t.id === selectedPreset) {
        return {
          ...t,
          prompt: customPrompt,
          aesthetic: targetAesthetic
        };
      }
      return t;
    });
    saveCustomTemplates();
    toast.success($locale === 'id' ? 'Perubahan disimpan!' : 'Changes saved!');
  }

  function handleDeleteTemplate() {
    if (!selectedPreset.startsWith('custom-')) return;
    const confirmDelete = confirm($locale === 'id' ? 'Hapus template ini?' : 'Delete this template?');
    if (!confirmDelete) return;
    
    customTemplates = customTemplates.filter(t => t.id !== selectedPreset);
    saveCustomTemplates();
    selectedPreset = 'technical';
    toast.success($locale === 'id' ? 'Template dihapus!' : 'Template deleted!');
  }

  function exportCsvReport() {
    if (logs.length === 0) {
      toast.error($locale === 'id' ? 'Tidak ada data pemilahan untuk diekspor!' : 'No culling data available to export!');
      return;
    }
    
    // Compile CSV headers
    let csvContent = 'data:text/csv;charset=utf-8,';
    csvContent += 'Timestamp,Filename,Status,Message\n';
    
    // Escape and add rows
    logs.forEach(log => {
      const escapedFilename = log.fileName ? `"${log.fileName.replace(/"/g, '""')}"` : '""';
      const escapedMsg = `"${log.message.replace(/"/g, '""')}"`;
      csvContent += `${log.timestamp},${escapedFilename},${log.status},${escapedMsg}\n`;
    });
    
    const encodedUri = encodeURI(csvContent);
    const link = document.createElement('a');
    
    const timestampStr = new Date().toISOString().replace(/[-:.]/g, '').slice(0, 15);
    link.setAttribute('href', encodedUri);
    link.setAttribute('download', `keepix_ai_cull_report_${timestampStr}.csv`);
    document.body.appendChild(link);
    
    link.click();
    document.body.removeChild(link);
    toast.success($locale === 'id' ? 'Laporan CSV berhasil diunduh!' : 'CSV Report downloaded successfully!');
  }

  function handleLogClick(log: CullLog) {
    if (!log.fileName) return;
    const items = get(mediaItems);
    const index = items.findIndex(item => item.file_name === log.fileName);
    if (index !== -1) {
      navigateTo(index);
      toast.info($locale === 'id' ? `Menampilkan foto: ${log.fileName}` : `Inspecting photo: ${log.fileName}`);
    } else {
      toast.error($locale === 'id' ? `Foto tidak ditemukan di workspace saat ini` : `Photo not found in current workspace`);
    }
  }

  async function fetchWithBackoff(url: string, options: RequestInit, retries = 3, delay = 2000): Promise<Response> {
    try {
      const response = await fetch(url, options);
      if (response.status === 429 && retries > 0) {
        addLog('', 'warning', `Rate limit hit (429). Retrying in ${delay / 1000}s...`);
        
        for (let countdown = delay / 1000; countdown > 0; countdown--) {
          if (stopRequested) break;
          await new Promise(r => setTimeout(r, 1000));
        }
        
        if (stopRequested) {
          throw new Error('Process stopped during backoff countdown');
        }
        
        return fetchWithBackoff(url, options, retries - 1, delay * 2);
      }
      return response;
    } catch (err) {
      if (retries > 0 && !stopRequested) {
        addLog('', 'warning', `Network error. Retrying in ${delay / 1000}s...`);
        await new Promise(r => setTimeout(r, delay));
        return fetchWithBackoff(url, options, retries - 1, delay * 2);
      }
      throw err;
    }
  }


  function addLog(fileName: string, status: CullLog['status'], message: string) {
    const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    logs = [...logs, { timestamp, fileName, status, message }];
    
    // Auto-scroll console
    setTimeout(() => {
      if (consoleContainer) {
        consoleContainer.scrollTop = consoleContainer.scrollHeight;
      }
    }, 50);
  }

  // Cancel trigger
  let stopRequested = false;

  async function startCulling() {
    if (!apiKey) {
      toast.error($locale === 'id' ? 'Masukkan Gemini API Key terlebih dahulu!' : 'Please enter your Gemini API Key first!');
      return;
    }

    saveConfig();
    stopRequested = false;
    isCulling = true;
    isPaused = false;
    logs = [];
    stats = { best: 0, trash: 0, draft: 0, review: 0, skipped: 0 };
    currentCullIndex = 0;
    startTime = Date.now();
    elapsedTime = 0;
    speed = 0;
    etr = '--';

    const allMedia = cullScope === 'all' ? get(mediaItems) : get(mediaItems).filter(item => !item.category_id);
    
    // Filter photos and IGNORE webp extensions case-insensitively
    const photos = allMedia.filter(item => {
      if (item.file_type !== 'photo') return false;
      const ext = item.file_name.split('.').pop()?.toLowerCase();
      return ext !== 'webp';
    });
    
    totalToCull = photos.length;

    if (totalToCull === 0) {
      toast.info($locale === 'id' ? 'Tidak ada foto (non-webp) yang perlu dipilah.' : 'No non-webp photos found matching selection scope.');
      isCulling = false;
      return;
    }

    addLog('', 'info', `Starting Agentic Auto-Cull. Total photos to analyze: ${totalToCull}`);

    for (let i = 0; i < photos.length; i++) {
      if (stopRequested) {
        addLog('', 'warning', 'Culling process stopped by user.');
        break;
      }

      while (isPaused) {
        await new Promise(r => setTimeout(r, 500));
        if (stopRequested) break;
      }
      if (stopRequested) break;

      currentCullIndex = i + 1;
      const photo = photos[i];
      currentItemName = photo.file_name;

      // Read base64
      try {
        const previewPath = photo.preview_path || photo.file_path;
        currentItemSrc = await invoke<string>('read_image_base64', { path: previewPath });
        
        // Load EXIF if available
        let exifStr = 'No EXIF metadata available';
        try {
          const exif = await invoke<any>('get_metadata', { mediaId: photo.id });
          if (exif) {
            exifStr = `Camera: ${exif.camera_model || 'Unknown'}, Lens: ${exif.lens_model || 'Unknown'}, Aperture: ${exif.aperture || 'Unknown'}, ISO: ${exif.iso || 'Unknown'}, Shutter: ${exif.shutter_speed || 'Unknown'}`;
          }
        } catch {}

        // Request Gemini
        const base64DataOnly = currentItemSrc.split(',')[1];
        const mimeType = currentItemSrc.split(';')[0].split(':')[1] || 'image/webp';
        
        const targetAestheticName = AESTHETIC_STYLES[targetAesthetic as keyof typeof AESTHETIC_STYLES]?.en || 'Natural Correction';

        let promptTemplate = `You are an expert AI photo-culling assistant built for professional photographers and editors.
Your goal is to analyze the provided photo alongside its EXIF metadata and categorize it based on the user's specific instructions.

### EXIF Metadata:
${exifStr}

### User Culling Instructions:
${customPrompt}

### Target Aesthetic Style:
You must tailor your adjustments to match this style: "${targetAestheticName}" (e.g., exposure corrections, contrast curves, white balance temperature/tint shifts, and color vibrancy adjustments appropriate for a "${targetAestheticName}" aesthetic).

### Rules for Output:
1. You must respond with a JSON object.
2. Category must be one of:
   - "best" (keep, high quality, strong expression, main shots)
   - "trash" (out of focus, duplicate test frames, blinking eyes, poorly exposed with no rescue value)
   - "draft" (secondary select, good but not the best, useful backups)
   - "review" (needs human inspection, ambiguous quality, interesting but flawed)
   - "none" (uncategorized)
3. Rating must be a number between 0 and 5 stars.
4. Color label must be one of: "red", "orange", "yellow", "green", or "none".
5. Provide a concise, clear reason for your decision (maximum 15 words) in the user's language (Indonesian if the instructions are in Indonesian, otherwise English).`;

        if (enableAutoEdit) {
          promptTemplate += `\n6. Recommend optimal non-destructive photo slider adjustments under the "autoEdit" key. Your goal is to auto-enhance the photo to correct lighting, exposure, contrast, white balance (temperature/tint), color richness (saturation/vibrance), detail (clarity/sharpening), and optional creative effects (vignette/grain) based on standard aesthetic guidelines for the style: "${targetAestheticName}".`;
        }

        promptTemplate += `\n\nResponse format:
{
  "category": "best" | "trash" | "draft" | "review" | "none",
  "rating": number,
  "colorLabel": "red" | "orange" | "yellow" | "green" | "none",
  "reason": "concise explanation"`;

        if (enableAutoEdit) {
          promptTemplate += `,
  "autoEdit": {
    "exposure": number,
    "contrast": number,
    "highlights": number,
    "shadows": number,
    "temperature": number,
    "tint": number,
    "saturation": number,
    "vibrance": number,
    "clarity": number,
    "vignette": number,
    "sharpen": number,
    "grain": number
  }`;
        }
        
        promptTemplate += `\n}`;

        // Schema construction
        const responseProperties: any = {
          category: { type: 'STRING', enum: ['trash', 'best', 'draft', 'review', 'none'] },
          rating: { type: 'INTEGER', minimum: 0, maximum: 5 },
          colorLabel: { type: 'STRING', enum: ['red', 'orange', 'yellow', 'green', 'none'] },
          reason: { type: 'STRING' }
        };

        if (enableAutoEdit) {
          responseProperties.autoEdit = {
            type: 'OBJECT',
            properties: {
              exposure: { type: 'INTEGER', minimum: -100, maximum: 100 },
              contrast: { type: 'INTEGER', minimum: -100, maximum: 100 },
              highlights: { type: 'INTEGER', minimum: -100, maximum: 100 },
              shadows: { type: 'INTEGER', minimum: -100, maximum: 100 },
              temperature: { type: 'INTEGER', minimum: -100, maximum: 100 },
              tint: { type: 'INTEGER', minimum: -100, maximum: 100 },
              saturation: { type: 'INTEGER', minimum: -100, maximum: 100 },
              vibrance: { type: 'INTEGER', minimum: -100, maximum: 100 },
              clarity: { type: 'INTEGER', minimum: -100, maximum: 100 },
              vignette: { type: 'INTEGER', minimum: 0, maximum: 100 },
              sharpen: { type: 'INTEGER', minimum: 0, maximum: 100 },
              grain: { type: 'INTEGER', minimum: 0, maximum: 100 }
            },
            required: ['exposure', 'contrast', 'highlights', 'shadows', 'temperature', 'tint', 'saturation', 'vibrance', 'clarity', 'vignette', 'sharpen', 'grain']
          };
        }

        const url = `https://generativelanguage.googleapis.com/v1beta/models/${modelName}:generateContent?key=${apiKey}`;
        const response = await fetchWithBackoff(url, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            contents: [{
              parts: [
                { text: promptTemplate },
                { inlineData: { mimeType, data: base64DataOnly } }
              ]
            }],
            generationConfig: {
              responseMimeType: 'application/json',
              responseSchema: {
                type: 'OBJECT',
                properties: responseProperties,
                required: enableAutoEdit ? ['category', 'reason', 'autoEdit'] : ['category', 'reason']
              }
            }
          })
        });

        if (!response.ok) {
          throw new Error(`API returned status ${response.status}`);
        }

        const data = await response.json();
        const responseText = data.candidates?.[0]?.content?.parts?.[0]?.text;
        if (!responseText) {
          throw new Error('Empty response from model');
        }

        const aiResult = JSON.parse(responseText);
        
        // Map category keys to SQLite integers
        let categoryId: number | null = null;
        if (aiResult.category === 'trash') { categoryId = 1; stats.trash++; }
        else if (aiResult.category === 'best') { categoryId = 2; stats.best++; }
        else if (aiResult.category === 'draft') { categoryId = 3; stats.draft++; }
        else if (aiResult.category === 'review') { categoryId = 4; stats.review++; }
        else { stats.skipped++; }

        // Apply changes
        const projectId = photo.project_id;
        await setCategoryForItem(photo.id, projectId, categoryId);

        if (typeof aiResult.rating === 'number') {
          await setStarRatingForItem(photo.id, aiResult.rating);
        }

        if (aiResult.colorLabel && aiResult.colorLabel !== 'none') {
          await setColorLabelForItem(photo.id, aiResult.colorLabel);
        }

        // Apply auto adjustments if enabled
        let editLogMsg = '';
        if (enableAutoEdit && aiResult.autoEdit) {
          const aiAdjustments = {
            ...DEFAULT_ADJUSTMENTS,
            exposure: aiResult.autoEdit.exposure ?? 0,
            contrast: aiResult.autoEdit.contrast ?? 0,
            highlights: aiResult.autoEdit.highlights ?? 0,
            shadows: aiResult.autoEdit.shadows ?? 0,
            temperature: aiResult.autoEdit.temperature ?? 0,
            tint: aiResult.autoEdit.tint ?? 0,
            saturation: aiResult.autoEdit.saturation ?? 0,
            vibrance: aiResult.autoEdit.vibrance ?? 0,
            clarity: aiResult.autoEdit.clarity ?? 0,
            vignette: aiResult.autoEdit.vignette ?? 0,
            sharpen: aiResult.autoEdit.sharpen ?? 0,
            grain: aiResult.autoEdit.grain ?? 0
          };

          await invoke('save_adjustments', {
            mediaId: photo.id,
            adjustmentsJson: JSON.stringify(aiAdjustments),
          });

          mediaItems.update(items =>
            items.map(item =>
              item.id === photo.id ? { ...item, adjustments: JSON.stringify(aiAdjustments) } : item
            )
          );

          editLogMsg = ` | Auto-Edits: Exp ${aiAdjustments.exposure > 0 ? '+' : ''}${aiAdjustments.exposure}, Temp ${aiAdjustments.temperature > 0 ? '+' : ''}${aiAdjustments.temperature}`;
        }

        const statusType = aiResult.category === 'best' ? 'success' :
                           aiResult.category === 'trash' ? 'error' :
                           aiResult.category === 'draft' ? 'info' : 'warning';

        addLog(photo.file_name, statusType, `${aiResult.category.toUpperCase()} (${aiResult.rating || 0}★) — ${aiResult.reason}${editLogMsg}`);

        // Update ETR & speed
        if (startTime) {
          const now = Date.now();
          elapsedTime = (now - startTime) / 1000;
          speed = elapsedTime / (i + 1);
          const remaining = totalToCull - (i + 1);
          const etrSeconds = Math.round(remaining * speed);
          if (etrSeconds < 60) {
            etr = `${etrSeconds}s`;
          } else {
            const mins = Math.floor(etrSeconds / 60);
            const secs = etrSeconds % 60;
            etr = `${mins}m ${secs}s`;
          }
        }

      } catch (err: any) {
        console.error('Failed to cull item:', err);
        addLog(photo.file_name, 'error', `Failed: ${err.message}`);
        stats.skipped++;
      }
    }

    isCulling = false;
    if (stopRequested) {
      toast.info($locale === 'id' ? 'Pemilahan otomatis AI dihentikan.' : 'AI auto-culling process stopped.');
    } else {
      toast.success($locale === 'id' ? 'Pemilahan otomatis AI selesai!' : 'AI auto-culling process completed!');
    }
    
    // Final database stats refresh
    const displayList = get(mediaItems);
    if (displayList.length > 0) {
      await loadCategoryStats(displayList[0].project_id);
    }
  }

  async function runSingleTest() {
    const photo = get(currentItem);
    if (!photo) {
      toast.error($locale === 'id' ? 'Pilih foto di workspace terlebih dahulu!' : 'Please select a photo in the workspace first!');
      return;
    }
    
    if (photo.file_type !== 'photo') {
      toast.error($locale === 'id' ? 'Item terpilih bukan berupa foto!' : 'Selected item is not a photo!');
      return;
    }
    
    const ext = photo.file_name.split('.').pop()?.toLowerCase();
    if (ext === 'webp') {
      toast.error($locale === 'id' ? 'Foto WebP diabaikan oleh AI!' : 'WebP photos are ignored by AI!');
      return;
    }
    
    if (!apiKey) {
      toast.error($locale === 'id' ? 'Masukkan Gemini API Key terlebih dahulu!' : 'Please enter your Gemini API Key first!');
      return;
    }
    
    saveConfig();
    stopRequested = false;
    isCulling = true;
    currentItemName = photo.file_name;
    
    addLog(photo.file_name, 'info', `[DRY RUN] Starting AI dry-run test on selected photo...`);
    
    try {
      const previewPath = photo.preview_path || photo.file_path;
      currentItemSrc = await invoke<string>('read_image_base64', { path: previewPath });
      
      // Load EXIF if available
      let exifStr = 'No EXIF metadata available';
      try {
        const exif = await invoke<any>('get_metadata', { mediaId: photo.id });
        if (exif) {
          exifStr = `Camera: ${exif.camera_model || 'Unknown'}, Lens: ${exif.lens_model || 'Unknown'}, Aperture: ${exif.aperture || 'Unknown'}, ISO: ${exif.iso || 'Unknown'}, Shutter: ${exif.shutter_speed || 'Unknown'}`;
        }
      } catch {}

      const base64DataOnly = currentItemSrc.split(',')[1];
      const mimeType = currentItemSrc.split(';')[0].split(':')[1] || 'image/webp';
      const targetAestheticName = AESTHETIC_STYLES[targetAesthetic as keyof typeof AESTHETIC_STYLES]?.en || 'Natural Correction';

      let promptTemplate = `You are an expert AI photo-culling assistant built for professional photographers and editors.
Your goal is to analyze the provided photo alongside its EXIF metadata and categorize it based on the user's specific instructions.

### EXIF Metadata:
${exifStr}

### User Culling Instructions:
${customPrompt}

### Target Aesthetic Style:
You must tailor your adjustments to match this style: "${targetAestheticName}" (e.g., exposure corrections, contrast curves, white balance temperature/tint shifts, and color vibrancy adjustments appropriate for a "${targetAestheticName}" aesthetic).

### Rules for Output:
1. You must respond with a JSON object.
2. Category must be one of:
   - "best" (keep, high quality, strong expression, main shots)
   - "trash" (out of focus, duplicate test frames, blinking eyes, poorly exposed with no rescue value)
   - "draft" (secondary select, good but not the best, useful backups)
   - "review" (needs human inspection, ambiguous quality, interesting but flawed)
   - "none" (uncategorized)
3. Rating must be a number between 0 and 5 stars.
4. Color label must be one of: "red", "orange", "yellow", "green", or "none".
5. Provide a concise, clear reason for your decision (maximum 15 words) in the user's language (Indonesian if the instructions are in Indonesian, otherwise English).`;

      if (enableAutoEdit) {
        promptTemplate += `\n6. Recommend optimal non-destructive photo slider adjustments under the "autoEdit" key. Your goal is to auto-enhance the photo to correct lighting, exposure, contrast, white balance (temperature/tint), color richness (saturation/vibrance), detail (clarity/sharpening), and optional creative effects (vignette/grain) based on standard aesthetic guidelines for the style: "${targetAestheticName}".`;
      }

      promptTemplate += `\n\nResponse format:
{
  "category": "best" | "trash" | "draft" | "review" | "none",
  "rating": number,
  "colorLabel": "red" | "orange" | "yellow" | "green" | "none",
  "reason": "concise explanation"`;

      if (enableAutoEdit) {
        promptTemplate += `,
  "autoEdit": {
    "exposure": number,
    "contrast": number,
    "highlights": number,
    "shadows": number,
    "temperature": number,
    "tint": number,
    "saturation": number,
    "vibrance": number,
    "clarity": number,
    "vignette": number,
    "sharpen": number,
    "grain": number
  }`;
      }
      
      promptTemplate += `\n}`;

      // Schema construction
      const responseProperties: any = {
        category: { type: 'STRING', enum: ['trash', 'best', 'draft', 'review', 'none'] },
        rating: { type: 'INTEGER', minimum: 0, maximum: 5 },
        colorLabel: { type: 'STRING', enum: ['red', 'orange', 'yellow', 'green', 'none'] },
        reason: { type: 'STRING' }
      };

      if (enableAutoEdit) {
        responseProperties.autoEdit = {
          type: 'OBJECT',
          properties: {
            exposure: { type: 'INTEGER', minimum: -100, maximum: 100 },
            contrast: { type: 'INTEGER', minimum: -100, maximum: 100 },
            highlights: { type: 'INTEGER', minimum: -100, maximum: 100 },
            shadows: { type: 'INTEGER', minimum: -100, maximum: 100 },
            temperature: { type: 'INTEGER', minimum: -100, maximum: 100 },
            tint: { type: 'INTEGER', minimum: -100, maximum: 100 },
            saturation: { type: 'INTEGER', minimum: -100, maximum: 100 },
            vibrance: { type: 'INTEGER', minimum: -100, maximum: 100 },
            clarity: { type: 'INTEGER', minimum: -100, maximum: 100 },
            vignette: { type: 'INTEGER', minimum: 0, maximum: 100 },
            sharpen: { type: 'INTEGER', minimum: 0, maximum: 100 },
            grain: { type: 'INTEGER', minimum: 0, maximum: 100 }
          },
          required: ['exposure', 'contrast', 'highlights', 'shadows', 'temperature', 'tint', 'saturation', 'vibrance', 'clarity', 'vignette', 'sharpen', 'grain']
        };
      }

      const url = `https://generativelanguage.googleapis.com/v1beta/models/${modelName}:generateContent?key=${apiKey}`;
      const response = await fetchWithBackoff(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          contents: [{
            parts: [
              { text: promptTemplate },
              { inlineData: { mimeType, data: base64DataOnly } }
            ]
          }],
          generationConfig: {
            responseMimeType: 'application/json',
            responseSchema: {
              type: 'OBJECT',
              properties: responseProperties,
              required: enableAutoEdit ? ['category', 'reason', 'autoEdit'] : ['category', 'reason']
            }
          }
        })
      });

      if (!response.ok) {
        throw new Error(`API returned status ${response.status}`);
      }

      const data = await response.json();
      const responseText = data.candidates?.[0]?.content?.parts?.[0]?.text;
      if (!responseText) {
        throw new Error('Empty response from model');
      }

      const aiResult = JSON.parse(responseText);
      
      let editLogMsg = '';
      if (enableAutoEdit && aiResult.autoEdit) {
        editLogMsg = `\n[DRY RUN EDIT] Recommended adjustments: Exp ${aiResult.autoEdit.exposure}, Contrast ${aiResult.autoEdit.contrast}, Highlights ${aiResult.autoEdit.highlights}, Shadows ${aiResult.autoEdit.shadows}, Temp ${aiResult.autoEdit.temperature}, Tint ${aiResult.autoEdit.tint}, Saturation ${aiResult.autoEdit.saturation}, Vibrance ${aiResult.autoEdit.vibrance}, Clarity ${aiResult.autoEdit.clarity}, Vignette ${aiResult.autoEdit.vignette}, Sharpen ${aiResult.autoEdit.sharpen}, Grain ${aiResult.autoEdit.grain}`;
      }

      addLog(photo.file_name, 'success', `[DRY RUN RESULT] Category: ${aiResult.category.toUpperCase()} (${aiResult.rating || 0}★, Color: ${aiResult.colorLabel || 'none'}) — Reason: ${aiResult.reason}${editLogMsg}`);
      
      toast.success($locale === 'id' ? 'Uji coba AI selesai (tidak disimpan ke database).' : 'AI dry-run completed (not saved to database).');
      
    } catch (err: any) {
      console.error('Dry run failed:', err);
      addLog(photo.file_name, 'error', `[DRY RUN FAILED]: ${err.message}`);
    } finally {
      isCulling = false;
    }
  }

  function handleClose() {
    if (isCulling) {
      const confirmClose = confirm($locale === 'id' ? 'Proses pemilahan AI sedang berjalan. Hentikan dan tutup?' : 'AI culling is running. Stop and close?');
      if (confirmClose) {
        stopRequested = true;
        showAiCull.set(false);
      }
    } else {
      showAiCull.set(false);
    }
  }
</script>

{#if $showAiCull}
  <div class="modal-overlay" role="presentation" onclick={handleClose}>
    <div class="modal-content glass-card ai-cull-modal" onclick={(e) => e.stopPropagation()} role="dialog">
      <!-- Modal Header -->
      <div class="modal-header">
        <div class="header-title">
          <span class="ai-icon">🤖</span>
          <h3>{$locale === 'id' ? 'Asisten Pemilahan AI' : 'AI Auto-Cull Assistant'}</h3>
        </div>
        <button class="btn-close" onclick={handleClose} title="Close">
          <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="4" y1="4" x2="14" y2="14"/>
            <line x1="14" y1="4" x2="4" y2="14"/>
          </svg>
        </button>
      </div>

      <!-- Split Layout -->
      <div class="ai-cull-layout">
        <!-- Configuration Side -->
        <div class="ai-cull-config">
          <div class="form-group">
            <label for="gemini-key">Gemini API Key</label>
            <input 
              type="password" 
              id="gemini-key" 
              placeholder="AIzaSy..." 
              bind:value={apiKey} 
              disabled={isCulling}
            />
            <span class="input-tip">
              {$locale === 'id' 
                ? 'Kunci disimpan secara lokal di browser Anda.' 
                : 'API key is stored locally in your browser.'}
            </span>
          </div>

          <div class="form-group">
            <label for="gemini-model">{$locale === 'id' ? 'Model AI' : 'AI Model'}</label>
            <select id="gemini-model" bind:value={modelName} disabled={isCulling}>
              <option value="gemini-2.5-flash">Gemini 2.5 Flash ({$locale === 'id' ? 'Cepat & Ekonomis' : 'Fast & Cost-efficient'})</option>
              <option value="gemini-2.5-pro">Gemini 2.5 Pro ({$locale === 'id' ? 'Detail & Akurat' : 'High Precision'})</option>
            </select>
          </div>

          <div class="form-group">
            <label for="cull-scope">{$locale === 'id' ? 'Cakupan Foto' : 'Culling Scope'}</label>
            <select id="cull-scope" bind:value={cullScope} disabled={isCulling}>
              <option value="uncategorized">{$locale === 'id' ? 'Foto Belum Dikategorikan' : 'Uncategorized Photos Only'}</option>
              <option value="all">{$locale === 'id' ? 'Semua Foto di Workspace' : 'All Photos in Workspace'}</option>
            </select>
          </div>

          <div class="form-group">
            <label for="cull-preset">{$locale === 'id' ? 'Aturan Template AI' : 'AI Rules Template'}</label>
            <select id="cull-preset" bind:value={selectedPreset} disabled={isCulling}>
              <optgroup label="System Presets">
                <option value="technical">{$locale === 'id' ? 'Kualitas Teknis & Ketajaman' : 'Technical Quality & Sharpness'}</option>
                <option value="portrait">{$locale === 'id' ? 'Foto Potret (Wajah & Ekspresi)' : 'Portrait Photography (Faces & Smiles)'}</option>
                <option value="landscape">{$locale === 'id' ? 'Foto Lanskap & Alam' : 'Landscape & Nature'}</option>
                <option value="event">{$locale === 'id' ? 'Foto Acara / Wedding' : 'Event / Wedding Moments'}</option>
              </optgroup>
              {#if customTemplates.length > 0}
                <optgroup label="Custom Templates">
                  {#each customTemplates as tmpl}
                    <option value={tmpl.id}>{tmpl.name}</option>
                  {/each}
                </optgroup>
              {/if}
              <optgroup label="Ad-hoc">
                <option value="custom">{$locale === 'id' ? 'Aturan Kustom Mandiri' : 'Custom Rules (Manual)'}</option>
              </optgroup>
            </select>
          </div>

          <!-- Custom Template Actions -->
          {#if selectedPreset.startsWith('custom-') || selectedPreset === 'custom'}
            <div class="template-actions-row">
              <button class="btn btn-secondary btn-template-action" onclick={handleCreateTemplate} title="Create New Template">
                <span>➕</span> {$locale === 'id' ? 'Baru' : 'New'}
              </button>
              {#if selectedPreset.startsWith('custom-')}
                <button class="btn btn-secondary btn-template-action" onclick={handleSaveTemplate} title="Save Changes">
                  <span>💾</span> {$locale === 'id' ? 'Simpan' : 'Save'}
                </button>
                <button class="btn btn-secondary btn-template-action danger" onclick={handleDeleteTemplate} title="Delete Template">
                  <span>🗑️</span> {$locale === 'id' ? 'Hapus' : 'Delete'}
                </button>
              {/if}
            </div>
          {:else}
            <div class="template-actions-row">
              <button class="btn btn-secondary btn-template-action" onclick={handleCreateTemplate} title="Create Custom Template">
                <span>➕</span> {$locale === 'id' ? 'Buat Kustom' : 'Save as Custom'}
              </button>
            </div>
          {/if}

          <div class="form-group-checkbox">
            <input 
              type="checkbox" 
              id="enable-auto-edit" 
              bind:checked={enableAutoEdit} 
              disabled={isCulling} 
            />
            <label for="enable-auto-edit">
              {$locale === 'id' 
                ? 'Edit & Sempurnakan Otomatis (AI Auto-Enhance)' 
                : 'Auto-Edit & Enhance (AI Auto-Enhance)'}
            </label>
          </div>

          {#if enableAutoEdit}
            <div class="form-group transition-aesthetic">
              <label for="target-aesthetic">{$locale === 'id' ? 'Gaya Estetika AI' : 'AI Aesthetic Style'}</label>
              <select id="target-aesthetic" bind:value={targetAesthetic} disabled={isCulling}>
                <option value="natural">{$locale === 'id' ? AESTHETIC_STYLES.natural.id : AESTHETIC_STYLES.natural.en}</option>
                <option value="cinematic">{$locale === 'id' ? AESTHETIC_STYLES.cinematic.id : AESTHETIC_STYLES.cinematic.en}</option>
                <option value="vintage">{$locale === 'id' ? AESTHETIC_STYLES.vintage.id : AESTHETIC_STYLES.vintage.en}</option>
                <option value="bw">{$locale === 'id' ? AESTHETIC_STYLES.bw.id : AESTHETIC_STYLES.bw.en}</option>
                <option value="punchy">{$locale === 'id' ? AESTHETIC_STYLES.punchy.id : AESTHETIC_STYLES.punchy.en}</option>
                <option value="none">{$locale === 'id' ? AESTHETIC_STYLES.none.id : AESTHETIC_STYLES.none.en}</option>
              </select>
            </div>
          {/if}

          <div class="form-group flex-fill">
            <label for="cull-instructions">{$locale === 'id' ? 'Panduan / Aturan Pemilahan' : 'Culling Rules / Criteria'}</label>
            <textarea
              id="cull-instructions"
              bind:value={customPrompt}
              disabled={isCulling}
              oninput={() => selectedPreset = 'custom'}
              placeholder="e.g. Keep photos where subjects are smiling..."
            ></textarea>
          </div>

          <!-- Controls -->
          <div class="cull-controls">
            {#if !isCulling}
              <div class="cull-start-buttons">
                <button class="btn btn-primary start-btn" onclick={startCulling}>
                  <span>▶</span>
                  <span>{$locale === 'id' ? 'Mulai Pemilahan AI' : 'Start Auto-Cull'}</span>
                </button>
                <button class="btn btn-secondary test-btn" onclick={runSingleTest} title="Test AI on selected photo">
                  <span>🔬</span>
                  <span>{$locale === 'id' ? 'Uji Coba' : 'Test AI'}</span>
                </button>
              </div>
            {:else}
              <div class="running-controls">
                <button class="btn btn-ghost" onclick={() => isPaused = !isPaused}>
                  <span>{isPaused ? '▶ Resume' : '⏸ Pause'}</span>
                </button>
                <button class="btn btn-primary danger-btn" onclick={() => stopRequested = true}>
                  <span>⏹ Stop</span>
                </button>
              </div>
            {/if}
          </div>
        </div>

        <!-- Dashboard / Running Console Side -->
        <div class="ai-cull-dashboard">
          <!-- Active Preview -->
          <div class="active-preview-container">
            {#if isCulling && currentItemSrc}
              <img src={currentItemSrc} alt="Analyzing..." class="active-preview-img" />
              <div class="active-preview-badge">
                <span class="pulse-dot"></span>
                <span>{$locale === 'id' ? 'Menganalisis:' : 'Analyzing:'} {currentItemName}</span>
              </div>
            {:else}
              <div class="active-preview-placeholder">
                <span>🤖</span>
                <p>
                  {isCulling 
                    ? ($locale === 'id' ? 'Memuat foto...' : 'Loading media asset...') 
                    : ($locale === 'id' ? 'Agent AI siap dijalankan' : 'AI Agent Ready to Start')}
                </p>
              </div>
            {/if}
          </div>

          <!-- Stats Grid -->
          <div class="stats-grid">
            <div class="stat-badge stat-best">
              <span class="label">BEST</span>
              <span class="value">{stats.best}</span>
            </div>
            <div class="stat-badge stat-draft">
              <span class="label">DRAFT</span>
              <span class="value">{stats.draft}</span>
            </div>
            <div class="stat-badge stat-review">
              <span class="label">REVIEW</span>
              <span class="value">{stats.review}</span>
            </div>
            <div class="stat-badge stat-trash">
              <span class="label">TRASH</span>
              <span class="value">{stats.trash}</span>
            </div>
          </div>

          <!-- Live Analytics Dashboard -->
          <div class="analytics-panel">
            <span class="panel-title">{$locale === 'id' ? 'KOMPOSISI PEMILAHAN AI' : 'AI CULLING COMPOSITION'}</span>
            <div class="segmented-progress-bar">
              {#if stats.best > 0}
                <div class="segment segment-best" style="width: {bestPct}%" title="Best: {stats.best} ({bestPct.toFixed(0)}%)"></div>
              {/if}
              {#if stats.draft > 0}
                <div class="segment segment-draft" style="width: {draftPct}%" title="Draft: {stats.draft} ({draftPct.toFixed(0)}%)"></div>
              {/if}
              {#if stats.review > 0}
                <div class="segment segment-review" style="width: {reviewPct}%" title="Review: {stats.review} ({reviewPct.toFixed(0)}%)"></div>
              {/if}
              {#if stats.trash > 0}
                <div class="segment segment-trash" style="width: {trashPct}%" title="Trash: {stats.trash} ({trashPct.toFixed(0)}%)"></div>
              {/if}
              {#if stats.skipped > 0}
                <div class="segment segment-skipped" style="width: {skippedPct}%" title="Skipped: {stats.skipped} ({skippedPct.toFixed(0)}%)"></div>
              {/if}
              {#if totalCount === 0}
                <div class="segment-empty">{$locale === 'id' ? 'Belum ada foto yang diproses' : 'No photos processed yet'}</div>
              {/if}
            </div>
            
            {#if totalCount > 0}
              <div class="legend-grid">
                <span class="legend-item text-best">🟢 Best: {bestPct.toFixed(0)}%</span>
                <span class="legend-item text-draft">🟡 Draft: {draftPct.toFixed(0)}%</span>
                <span class="legend-item text-review">🔵 Review: {reviewPct.toFixed(0)}%</span>
                <span class="legend-item text-trash">🔴 Trash: {trashPct.toFixed(0)}%</span>
              </div>
            {/if}
          </div>

          <!-- Progress Bar & ETR -->
          {#if isCulling}
            <div class="progress-section">
              <div class="progress-label">
                <span>{$locale === 'id' ? 'Kemajuan Pemilahan' : 'Progress'}</span>
                <span>{currentCullIndex} / {totalToCull} ({Math.round((currentCullIndex/totalToCull)*100)}%)</span>
              </div>
              <div class="progress-bar">
                <div class="progress-bar-fill" style="width: {(currentCullIndex/totalToCull)*100}%"></div>
              </div>
              <div class="progress-meta">
                <span>Speed: {speed > 0 ? speed.toFixed(1) : '--'}s/photo</span>
                <span>ETR: {etr}</span>
              </div>
            </div>
          {/if}

          <!-- Agent Console Log -->
          <div class="console-log-header">
            <span>Agentic Reasoning Console</span>
            <div class="log-header-right">
              {#if logs.length > 0}
                <button class="export-btn" onclick={exportCsvReport} title="Export CSV Report">
                  <span>📥</span> {$locale === 'id' ? 'Ekspor CSV' : 'Export CSV'}
                </button>
              {/if}
              <div class="log-filters">
                <button class="filter-chip" class:active={logFilter === 'all'} onclick={() => logFilter = 'all'}>ALL</button>
                <button class="filter-chip" class:active={logFilter === 'best'} onclick={() => logFilter = 'best'}>BEST</button>
                <button class="filter-chip" class:active={logFilter === 'trash'} onclick={() => logFilter = 'trash'}>TRASH</button>
                <button class="filter-chip" class:active={logFilter === 'warning'} onclick={() => logFilter = 'warning'}>WARN/ERR</button>
              </div>
            </div>
          </div>
          <div class="console-log-body" bind:this={consoleContainer}>
            {#if filteredLogs.length === 0}
              <div class="console-empty">
                <span>Waiting for agent action...</span>
              </div>
            {:else}
              {#each filteredLogs as log}
                <div 
                  class="log-row status-{log.status} interactive-log"
                  onclick={() => handleLogClick(log)}
                  onkeydown={(e) => e.key === 'Enter' && handleLogClick(log)}
                  role="button"
                  tabindex="0"
                  title={$locale === 'id' ? 'Klik untuk tampilkan di editor' : 'Click to inspect in editor'}
                >
                  <span class="log-time">[{log.timestamp}]</span>
                  {#if log.fileName}
                    <span class="log-file">{log.fileName}:</span>
                  {/if}
                  <span class="log-msg">{log.message}</span>
                </div>
              {/each}
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .ai-cull-modal {
    max-width: 900px !important;
    width: 95% !important;
    height: 640px;
    display: flex;
    flex-direction: column;
    padding: 0 !important;
    background: var(--bg-secondary) !important;
    border: 1px solid var(--border-strong) !important;
    box-shadow: var(--shadow-lg) !important;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-4) var(--space-5);
    border-bottom: 1px solid var(--border-subtle);
    background: rgba(255, 255, 255, 0.01);
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .header-title h3 {
    margin: 0;
    font-size: var(--text-md);
    font-weight: 600;
    color: var(--text-primary);
  }

  .ai-icon {
    font-size: 18px;
  }

  .btn-close {
    background: none;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: var(--space-1);
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
  }

  .btn-close:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.05);
  }

  /* Split Layout */
  .ai-cull-layout {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .ai-cull-config {
    width: 40%;
    border-right: 1px solid var(--border-subtle);
    padding: var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    background: rgba(0, 0, 0, 0.15);
    overflow-y: auto;
  }

  .ai-cull-dashboard {
    width: 60%;
    padding: var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    overflow: hidden;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex-shrink: 0;
  }

  .form-group label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
  }

  .form-group input, .form-group select, .form-group textarea {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    outline: none;
    transition: all 0.15s;
  }

  .form-group input:focus, .form-group select:focus, .form-group textarea:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent-soft);
  }

  /* Premium Checkbox styling */
  .form-group-checkbox {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    background: rgba(168, 85, 247, 0.06);
    border: 1px dashed rgba(168, 85, 247, 0.25);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all 0.15s ease-in-out;
    flex-shrink: 0;
  }

  .form-group-checkbox:hover {
    background: rgba(168, 85, 247, 0.12);
    border-color: rgba(168, 85, 247, 0.4);
  }

  .form-group-checkbox input[type="checkbox"] {
    accent-color: #a855f7;
    width: 15px;
    height: 15px;
    cursor: pointer;
  }

  .form-group-checkbox label {
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--text-primary);
    cursor: pointer;
    user-select: none;
  }

  .transition-aesthetic {
    animation: fadeIn 0.2s ease-in-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-5px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .form-group textarea {
    resize: none;
    flex: 1;
    min-height: 80px;
    font-size: var(--text-xs);
    line-height: 1.4;
  }

  .flex-fill {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .input-tip {
    font-size: 10px;
    color: var(--text-tertiary);
  }

  .cull-controls {
    margin-top: auto;
    padding-top: var(--space-2);
    flex-shrink: 0;
  }

  .start-btn {
    width: 100%;
    background: var(--accent);
    color: white;
    font-weight: 600;
    height: 38px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    box-shadow: var(--shadow-glow);
  }

  .start-btn:hover {
    background: var(--accent-hover);
  }

  .running-controls {
    display: flex;
    gap: var(--space-2);
  }

  .running-controls button {
    flex: 1;
    height: 38px;
    font-weight: 600;
  }

  .danger-btn {
    background: var(--color-buang) !important;
    border-color: transparent !important;
  }

  .danger-btn:hover {
    background: #dc2626 !important;
  }

  /* Dashboard details */
  .active-preview-container {
    height: 140px;
    background: #000;
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-subtle);
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }

  .active-preview-img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }

  .active-preview-badge {
    position: absolute;
    bottom: 8px;
    left: 8px;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    padding: 3px 8px;
    border-radius: 12px;
    font-size: 10px;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    gap: 6px;
    max-width: 90%;
  }

  .pulse-dot {
    width: 6px;
    height: 6px;
    background: #10b981;
    border-radius: 50%;
    animation: pulse 1.5s infinite;
  }

  .active-preview-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-tertiary);
  }

  .active-preview-placeholder span {
    font-size: 32px;
    opacity: 0.4;
  }

  .active-preview-placeholder p {
    font-size: var(--text-sm);
    margin: 0;
  }

  /* Stats badges */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-2);
  }

  .stat-badge {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-2);
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
  }

  .stat-badge .label {
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.5px;
    margin-bottom: 2px;
  }

  .stat-badge .value {
    font-family: var(--font-mono);
    font-size: var(--text-lg);
    font-weight: 700;
  }

  .stat-best {
    border-color: rgba(34, 197, 94, 0.2);
    color: var(--color-simpan);
  }

  .stat-draft {
    border-color: rgba(245, 158, 11, 0.2);
    color: var(--color-draft);
  }

  .stat-review {
    border-color: rgba(59, 130, 246, 0.2);
    color: var(--color-review);
  }

  .stat-trash {
    border-color: rgba(239, 68, 68, 0.2);
    color: var(--color-buang);
  }

  /* Progress section */
  .progress-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .progress-label {
    display: flex;
    justify-content: space-between;
    font-size: var(--text-xs);
    color: var(--text-secondary);
  }

  .progress-bar {
    width: 100%;
    height: 6px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--accent);
    border-radius: var(--radius-sm);
    transition: width 0.3s ease;
  }

  .progress-meta {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--text-tertiary);
  }

  /* Console logs */
  .console-log-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: 6px;
    margin-top: var(--space-2);
  }

  .console-log-header span {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-tertiary);
  }

  .log-filters {
    display: flex;
    gap: 4px;
  }

  .filter-chip {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-tertiary);
    font-family: var(--font-sans);
    font-size: 9px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .filter-chip:hover {
    border-color: var(--text-tertiary);
    color: var(--text-primary);
  }

  .filter-chip.active {
    background: var(--accent-soft);
    border-color: var(--accent);
    color: white;
  }

  .console-log-body {
    flex: 1;
    background: #09090b;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: var(--space-3);
    overflow-y: auto;
    font-family: var(--font-mono);
    font-size: 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .console-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-tertiary);
  }

  .log-row {
    display: flex;
    gap: 6px;
    line-height: 1.4;
    word-break: break-all;
  }

  .log-time {
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .log-file {
    color: var(--text-secondary);
    font-weight: 500;
    flex-shrink: 0;
  }

  .status-success .log-msg { color: var(--color-simpan); }
  .status-error .log-msg { color: var(--color-buang); }
  .status-info .log-msg { color: var(--text-primary); }
  .status-warning .log-msg { color: var(--color-review); }

  /* Expert Mode additions */
  .template-actions-row {
    display: flex;
    gap: var(--space-2);
    margin-top: calc(-1 * var(--space-2));
    flex-shrink: 0;
  }

  .btn-template-action {
    flex: 1;
    font-size: 10px !important;
    padding: var(--space-1) var(--space-2) !important;
    height: 26px !important;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 0.15s;
  }

  .btn-template-action:hover {
    background: var(--surface-card-hover);
    color: var(--text-primary);
    border-color: var(--text-tertiary);
  }

  .btn-template-action.danger:hover {
    background: rgba(239, 68, 68, 0.15);
    color: var(--color-buang);
    border-color: rgba(239, 68, 68, 0.3);
  }

  .cull-start-buttons {
    display: flex;
    gap: var(--space-2);
    width: 100%;
  }

  .cull-start-buttons .start-btn {
    flex: 2;
  }

  .cull-start-buttons .test-btn {
    flex: 1;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
    font-weight: 600;
    height: 38px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    cursor: pointer;
    transition: all 0.15s;
  }

  .cull-start-buttons .test-btn:hover {
    background: var(--surface-card-hover);
    border-color: var(--text-tertiary);
  }

  /* Live Analytics Panel */
  .analytics-panel {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: var(--space-3);
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex-shrink: 0;
  }

  .panel-title {
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.8px;
    color: var(--text-tertiary);
  }

  .segmented-progress-bar {
    width: 100%;
    height: 10px;
    background: var(--bg-tertiary);
    border-radius: var(--radius-full);
    overflow: hidden;
    display: flex;
    border: 1px solid var(--border-subtle);
  }

  .segment {
    height: 100%;
    transition: width 0.3s ease;
  }

  .segment-best { background: var(--color-simpan); }
  .segment-draft { background: var(--color-draft); }
  .segment-review { background: var(--color-review); }
  .segment-trash { background: var(--color-buang); }
  .segment-skipped { background: var(--text-tertiary); }

  .segment-empty {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 9px;
    color: var(--text-tertiary);
    background: transparent;
  }

  .legend-grid {
    display: flex;
    justify-content: space-between;
    font-size: 9px;
    font-weight: 600;
    gap: var(--space-2);
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .text-best { color: var(--color-simpan); }
  .text-draft { color: var(--color-draft); }
  .text-review { color: var(--color-review); }
  .text-trash { color: var(--color-buang); }

  /* Export Button */
  .log-header-right {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .export-btn {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 9px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    transition: all 0.15s;
    height: 19px;
  }

  .export-btn:hover {
    color: var(--text-primary);
    border-color: var(--text-tertiary);
    background: var(--surface-card-hover);
  }

  /* Interactive Logs styling */
  .interactive-log {
    cursor: pointer;
    transition: background 0.15s, transform 0.1s;
    border-radius: 2px;
    padding: 1px 4px;
  }

  .interactive-log:hover {
    background: rgba(255, 255, 255, 0.05);
    transform: translateX(2px);
  }

  .interactive-log:active {
    transform: translateX(0);
  }
</style>
