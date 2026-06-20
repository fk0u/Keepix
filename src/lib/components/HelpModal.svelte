<script lang="ts">
  import { locale } from '$lib/i18n';
  import { onMount } from 'svelte';
  
  let {
    show = true,
    onClose,
  }: {
    show?: boolean;
    onClose: () => void;
  } = $props();

  let activeTab = $state('start'); // 'start' | 'editing' | 'video' | 'shortcuts' | 'trouble'

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="modal-backdrop" onclick={onClose} role="presentation">
  <div class="modal-container glass-card" onclick={(e) => e.stopPropagation()} role="presentation">
    
    <!-- Sidebar Navigation -->
    <div class="modal-sidebar">
      <div class="sidebar-header">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="help-icon">
          <circle cx="12" cy="12" r="10"/>
          <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/>
          <line x1="12" y1="17" x2="12.01" y2="17"/>
        </svg>
        <span class="sidebar-title">{$locale === 'id' ? 'Bantuan & Dok' : 'Help & Doc'}</span>
      </div>
      <div class="sidebar-menu">
        <button class="menu-btn" class:active={activeTab === 'start'} onclick={() => activeTab = 'start'}>
          {$locale === 'id' ? 'Memulai Cepat' : 'Quick Start'}
        </button>
        <button class="menu-btn" class:active={activeTab === 'editing'} onclick={() => activeTab = 'editing'}>
          {$locale === 'id' ? 'Pengeditan Foto' : 'Photo Editing'}
        </button>
        <button class="menu-btn" class:active={activeTab === 'video'} onclick={() => activeTab = 'video'}>
          {$locale === 'id' ? 'Kompatibilitas Video' : 'Video Compatibility'}
        </button>
        <button class="menu-btn" class:active={activeTab === 'shortcuts'} onclick={() => activeTab = 'shortcuts'}>
          {$locale === 'id' ? 'Pintasan Keyboard' : 'Keyboard Shortcuts'}
        </button>
        <button class="menu-btn" class:active={activeTab === 'trouble'} onclick={() => activeTab = 'trouble'}>
          {$locale === 'id' ? 'Penyelesaian Masalah' : 'Troubleshooting'}
        </button>
      </div>
      <div class="sidebar-footer">
        <span>Keepix v4.0.1</span>
      </div>
    </div>

    <!-- Content Area -->
    <div class="modal-content">
      <button class="close-btn" onclick={onClose} aria-label="Tutup Bantuan">✕</button>

      {#if activeTab === 'start'}
        <div class="content-tab">
          {#if $locale === 'id'}
            <h2>🚀 Panduan Memulai Cepat</h2>
            <p class="tab-subtitle">Cara cepat memilah ribuan foto & video dengan Keepix.</p>
            
            <div class="step-card">
              <div class="step-num">1</div>
              <div class="step-body">
                <h3>Buat Proyek Baru</h3>
                <p>Klik <strong>Berkas &gt; Workspace Baru (Ctrl+N)</strong> dan pilih folder yang berisi foto/video Anda. Keepix akan memindai seluruh file secara lokal.</p>
              </div>
            </div>

            <div class="step-card">
              <div class="step-num">2</div>
              <div class="step-body">
                <h3>Lakukan Culling (Pilah Berkas)</h3>
                <p>Gunakan tombol panah <strong>Kiri/Kanan</strong> untuk navigasi. Berikan penilaian menggunakan hotkey cepat:</p>
                <ul>
                  <li><kbd>1</kbd> untuk menandai <strong>Buang (Trash)</strong></li>
                  <li><kbd>2</kbd> untuk menandai <strong>Simpan (Keep/Star)</strong></li>
                  <li><kbd>5</kbd> untuk bintang 5, <kbd>0</kbd> untuk reset bintang</li>
                  <li><kbd>6</kbd> - <kbd>9</kbd> untuk label warna</li>
                </ul>
              </div>
            </div>

            <div class="step-card">
              <div class="step-num">3</div>
              <div class="step-body">
                <h3>Ekspor Hasil Pemilahan</h3>
                <p>Klik <strong>Ekspor (Ctrl+E)</strong> untuk memindahkan, menyalin, atau mencantumkan daftar berkas yang sudah dipilah sesuai dengan kategori pilihan Anda ke folder eksternal.</p>
              </div>
            </div>
          {:else}
            <h2>🚀 Quick Start Guide</h2>
            <p class="tab-subtitle">How to cull thousands of photos & videos quickly with Keepix.</p>
            
            <div class="step-card">
              <div class="step-num">1</div>
              <div class="step-body">
                <h3>Create New Workspace</h3>
                <p>Click <strong>File &gt; New Workspace (Ctrl+N)</strong> and select the directory containing your media. Keepix will scan files locally.</p>
              </div>
            </div>

            <div class="step-card">
              <div class="step-num">2</div>
              <div class="step-body">
                <h3>Cull Media Items</h3>
                <p>Use <strong>Left/Right</strong> arrow keys to navigate. Grade and cull using fast hotkeys:</p>
                <ul>
                  <li><kbd>1</kbd> to mark as <strong>Trash</strong></li>
                  <li><kbd>2</kbd> to mark as <strong>Keep (Star)</strong></li>
                  <li><kbd>5</kbd> for 5-star rating, <kbd>0</kbd> to clear stars</li>
                  <li><kbd>6</kbd> - <kbd>9</kbd> for color labels</li>
                </ul>
              </div>
            </div>

            <div class="step-card">
              <div class="step-num">3</div>
              <div class="step-body">
                <h3>Export Culled Media</h3>
                <p>Click <strong>Export (Ctrl+E)</strong> to copy, move, or list selected files to your target storage folder based on assigned categories.</p>
              </div>
            </div>
          {/if}
        </div>
      {/if}

      {#if activeTab === 'editing'}
        <div class="content-tab">
          {#if $locale === 'id'}
            <h2>🎨 Panduan Pengeditan Foto</h2>
            <p class="tab-subtitle">Gunakan fitur edit profesional bawaan Keepix untuk menyempurnakan foto Anda.</p>

            <div class="step-card">
              <div class="step-num">1</div>
              <div class="step-body">
                <h3>Preset Instan</h3>
                <p>Pilih dari <strong>13 preset profesional</strong> seperti Sinematik, Vintage, Film Noir, Senja Hangat, dan lainnya. Setiap preset langsung menerapkan kombinasi penyesuaian yang telah dikurasi.</p>
              </div>
            </div>

            <div class="step-card">
              <div class="step-num">2</div>
              <div class="step-body">
                <h3>Penyesuaian Manual</h3>
                <p>Atur <strong>Pencahayaan, Kontras, Sorotan, Bayangan, Suhu, Saturasi, Ketajaman, Kejelasan, Vignette</strong>, dan <strong>Butiran Film</strong> menggunakan slider presisi. Semua perubahan bersifat non-destruktif.</p>
              </div>
            </div>

            <div class="step-card">
              <div class="step-num">3</div>
              <div class="step-body">
                <h3>Perbandingan Sebelum / Sesudah</h3>
                <p>Tekan <kbd>B</kbd> untuk mengaktifkan mode perbandingan. Geser pembatas untuk melihat perbedaan antara foto asli dan hasil edit. Tersedia mode <strong>Geser</strong> dan <strong>Berdampingan</strong>.</p>
              </div>
            </div>

            <div class="step-card">
              <div class="step-num">4</div>
              <div class="step-body">
                <h3>Alat Penyembuhan Titik (Healing)</h3>
                <p>Aktifkan Spot Healing dari panel Alat. <strong>Alt+Klik</strong> untuk menentukan titik sumber, lalu lukis di area yang ingin diperbaiki. Piksel akan disalin dari sumber dengan pencampuran halus.</p>
              </div>
            </div>

            <div class="step-card highlight">
              <div class="step-num">💡</div>
              <div class="step-body">
                <h3>Salin & Tempel Penyesuaian</h3>
                <p>Gunakan <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>C</kbd> untuk menyalin pengaturan edit, lalu navigasi ke foto lain dan tekan <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>V</kbd> untuk menerapkan pengaturan yang sama.</p>
              </div>
            </div>
          {:else}
            <h2>🎨 Photo Editing Guide</h2>
            <p class="tab-subtitle">Use Keepix's built-in professional editing features to enhance your photos.</p>

            <div class="step-card">
              <div class="step-num">1</div>
              <div class="step-body">
                <h3>Instant Presets</h3>
                <p>Choose from <strong>13 professional presets</strong> including Cinematic, Vintage, Film Noir, Warm Sunset, and more. Each preset instantly applies a curated combination of adjustments.</p>
              </div>
            </div>

            <div class="step-card">
              <div class="step-num">2</div>
              <div class="step-body">
                <h3>Manual Adjustments</h3>
                <p>Fine-tune <strong>Exposure, Contrast, Highlights, Shadows, Temperature, Saturation, Sharpen, Clarity, Vignette</strong>, and <strong>Film Grain</strong> with precision sliders. All changes are non-destructive.</p>
              </div>
            </div>

            <div class="step-card">
              <div class="step-num">3</div>
              <div class="step-body">
                <h3>Before / After Comparison</h3>
                <p>Press <kbd>B</kbd> to activate comparison mode. Drag the divider to see the difference between original and edited versions. Both <strong>Slider</strong> and <strong>Side-by-Side</strong> modes available.</p>
              </div>
            </div>

            <div class="step-card">
              <div class="step-num">4</div>
              <div class="step-body">
                <h3>Spot Healing Tool</h3>
                <p>Activate Spot Healing from the Tools panel. <strong>Alt+Click</strong> to set a source point, then paint over the area you want to fix. Pixels are cloned from the source with smooth feathered blending.</p>
              </div>
            </div>

            <div class="step-card highlight">
              <div class="step-num">💡</div>
              <div class="step-body">
                <h3>Copy & Paste Adjustments</h3>
                <p>Use <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>C</kbd> to copy edit settings, then navigate to another photo and press <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>V</kbd> to apply the same settings.</p>
              </div>
            </div>
          {/if}
        </div>
      {/if}

      {#if activeTab === 'video'}
        <div class="content-tab">
          {#if $locale === 'id'}
            <h2>📹 Kompatibilitas & Dukungan Video</h2>
            <p class="tab-subtitle">Informasi mengenai dukungan format MP4, MOV, dan MKV di Keepix.</p>
            
            <div class="info-block warning">
              <h3>Bagaimana Format Video Berjalan?</h3>
              <p>Keepix mendukung peninjauan langsung video format <strong>MP4</strong>, <strong>MOV</strong>, dan <strong>MKV</strong>. Pemutaran video menggunakan akselerasi perangkat keras sistem operasi Windows melalui WebView2.</p>
            </div>

            <div class="format-grid">
              <div class="format-card active">
                <span class="format-name">MP4 / H.264</span>
                <span class="status-ok">Didukung Penuh</span>
                <p>Berjalan sangat lancar dengan decoder bawaan.</p>
              </div>
              <div class="format-card active">
                <span class="format-name">MOV / ProRes / H.264</span>
                <span class="status-ok">Didukung</span>
                <p>Berjalan lancar jika menggunakan codec H.264/AAC.</p>
              </div>
              <div class="format-card warning">
                <span class="format-name">MKV / H.265 (HEVC)</span>
                <span class="status-warn">Perlu Ekstensi</span>
                <p>Membutuhkan ekstensi codec HEVC di Windows.</p>
              </div>
            </div>

            <div class="step-card highlight">
              <div class="step-num">💡</div>
              <div class="step-body">
                <h3>Cara Mengaktifkan Dukungan H.265 / HEVC:</h3>
                <p>Jika video H.265 (HEVC) atau file MKV Anda menampilkan layar hitam atau peringatan <em>"Video preview limited"</em>, silakan cari dan pasang <strong>"HEVC Video Extensions"</strong> secara gratis dari Microsoft Store.</p>
                <p>Ini memungkinkan WebView2 sistem menggunakan kartu grafis Anda untuk memecahkan sandi video H.265 secara langsung.</p>
              </div>
            </div>
          {:else}
            <h2>📹 Video Compatibility & Codecs</h2>
            <p class="tab-subtitle">Details about MP4, MOV, and MKV video support in Keepix.</p>
            
            <div class="info-block warning">
              <h3>How Video Playback Works</h3>
              <p>Keepix supports inline preview and playback of <strong>MP4</strong>, <strong>MOV</strong>, and <strong>MKV</strong> containers. It utilizes hardware-accelerated decoding via Windows WebView2.</p>
            </div>

            <div class="format-grid">
              <div class="format-card active">
                <span class="format-name">MP4 / H.264</span>
                <span class="status-ok">Fully Supported</span>
                <p>Plays smoothly with native web decoders.</p>
              </div>
              <div class="format-card active">
                <span class="format-name">MOV / ProRes / H.264</span>
                <span class="status-ok">Supported</span>
                <p>Works out-of-the-box if using H.264/AAC codecs.</p>
              </div>
              <div class="format-card warning">
                <span class="format-name">MKV / H.265 (HEVC)</span>
                <span class="status-warn">Needs Extension</span>
                <p>Requires HEVC video decoding extensions on Windows.</p>
              </div>
            </div>

            <div class="step-card highlight">
              <div class="step-num">💡</div>
              <div class="step-body">
                <h3>Enabling H.265 / HEVC Playback:</h3>
                <p>If H.265 videos or MKV files show a black screen or trigger the <em>"Video preview limited"</em> warning, install the <strong>"HEVC Video Extensions"</strong> package from the Microsoft Store.</p>
                <p>This links the native GPU hardware decoder into the WebView2 container, enabling instant high-res playback.</p>
              </div>
            </div>
          {/if}
        </div>
      {/if}

      {#if activeTab === 'shortcuts'}
        <div class="content-tab">
          <h2>⌨️ {$locale === 'id' ? 'Pintasan Keyboard (Cheat Sheet)' : 'Keyboard Shortcuts Cheat Sheet'}</h2>
          <p class="tab-subtitle">{$locale === 'id' ? 'Kuasai seluruh navigasi Keepix tanpa menyentuh mouse.' : 'Master Keepix culling workflows using only your keyboard.'}</p>

          <table class="shortcuts-table">
            <thead>
              <tr>
                <th>{$locale === 'id' ? 'Tombol' : 'Key'}</th>
                <th>{$locale === 'id' ? 'Fungsi / Deskripsi' : 'Action / Description'}</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td><kbd>Space</kbd></td>
                <td>{$locale === 'id' ? 'Ganti mode Tampilan (Grid / Pratinjau)' : 'Toggle View Mode (Grid / Preview)'}</td>
              </tr>
              <tr>
                <td><kbd>→</kbd> / <kbd>←</kbd></td>
                <td>{$locale === 'id' ? 'Navigasi ke Media Selanjutnya / Sebelumnya' : 'Go to Next / Previous Media item'}</td>
              </tr>
              <tr>
                <td><kbd>1</kbd> , <kbd>2</kbd> , <kbd>3</kbd> , <kbd>4</kbd></td>
                <td>{$locale === 'id' ? 'Beri Kategori (1: Buang, 2: Simpan, 3: Draft, 4: Review)' : 'Assign Category (1: Trash, 2: Keep, 3: Draft, 4: Review)'}</td>
              </tr>
              <tr>
                <td><kbd>5</kbd> / <kbd>0</kbd></td>
                <td>{$locale === 'id' ? 'Beri Rating Bintang 5 / Reset Rating' : 'Set 5-Star Rating / Reset Rating to 0'}</td>
              </tr>
              <tr>
                <td><kbd>6</kbd> , <kbd>7</kbd> , <kbd>8</kbd> , <kbd>9</kbd></td>
                <td>{$locale === 'id' ? 'Beri Label Warna (6: Merah, 7: Oranye, 8: Kuning, 9: Hijau)' : 'Assign Color Label (6: Red, 7: Orange, 8: Yellow, 9: Green)'}</td>
              </tr>
              <tr>
                <td><kbd>Ctrl</kbd> + <kbd>Z</kbd></td>
                <td>{$locale === 'id' ? 'Membatalkan Aksi Terakhir (Undo)' : 'Undo Last Action'}</td>
              </tr>
              <tr>
                <td><kbd>I</kbd></td>
                <td>{$locale === 'id' ? 'Buka/Tutup Panel Info Metadata' : 'Toggle Metadata Info Panel'}</td>
              </tr>
              <tr>
                <td><kbd>H</kbd></td>
                <td>{$locale === 'id' ? 'Tampilkan/Sembunyikan Histogram Foto' : 'Toggle Image Histogram Overlay'}</td>
              </tr>
              <tr>
                <td><kbd>O</kbd></td>
                <td>{$locale === 'id' ? 'Siklus Mode Analisis Fokus (Peaking / Zebra)' : 'Cycle Focus Diagnostics Overlay (Peaking / Zebra)'}</td>
              </tr>
              <tr>
                <td><kbd>[</kbd> / <kbd>J</kbd></td>
                <td>{$locale === 'id' ? 'Mundur 1 Bingkai Video (Frame Step Back)' : 'Step Back 1 Video Frame (~0.03s)'}</td>
              </tr>
              <tr>
                <td><kbd>]</kbd> / <kbd>L</kbd></td>
                <td>{$locale === 'id' ? 'Maju 1 Bingkai Video (Frame Step Forward)' : 'Step Forward 1 Video Frame (~0.03s)'}</td>
              </tr>
              <tr>
                <td><kbd>M</kbd></td>
                <td>{$locale === 'id' ? 'Bisukan / Bunyikan Suara Video (Mute)' : 'Mute / Unmute Video Audio'}</td>
              </tr>
              <tr>
                <td><kbd>Esc</kbd></td>
                <td>{$locale === 'id' ? 'Kembali ke Beranda / Tutup Dialog' : 'Go back to Home / Close dialogs'}</td>
              </tr>
              <tr>
                <td><kbd>B</kbd></td>
                <td>{$locale === 'id' ? 'Buka/Tutup Perbandingan Sebelum/Sesudah' : 'Toggle Before/After Comparison'}</td>
              </tr>
              <tr>
                <td><kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>C</kbd></td>
                <td>{$locale === 'id' ? 'Salin Penyesuaian Edit ke Clipboard' : 'Copy Edit Adjustments to Clipboard'}</td>
              </tr>
              <tr>
                <td><kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>V</kbd></td>
                <td>{$locale === 'id' ? 'Tempel Penyesuaian Edit dari Clipboard' : 'Paste Edit Adjustments from Clipboard'}</td>
              </tr>
            </tbody>
          </table>
        </div>
      {/if}

      {#if activeTab === 'trouble'}
        <div class="content-tab">
          {#if $locale === 'id'}
            <h2>🛠️ Penyelesaian Masalah</h2>
            <p class="tab-subtitle">Solusi untuk kendala umum yang terjadi di Keepix.</p>

            <div class="trouble-item">
              <h3>1. Video Berwarna Hitam atau Bersuara Saja</h3>
              <p>Ini biasanya terjadi jika format video dikodekan menggunakan codec HEVC/H.265. Silakan ikuti petunjuk di tab <strong>Kompatibilitas Video</strong> untuk mengunduh modul ekstensi decoder HEVC dari Microsoft.</p>
            </div>

            <div class="trouble-item">
              <h3>2. Antarmuka Terasa Lambat (Lags)</h3>
              <p>Keepix menggunakan RAM & GPU secara dinamis untuk rendering gambar RAW & Video resolusi tinggi. Anda dapat mengubah jumlah utas prosesor (CPU Threads) pada menu <strong>Sunting &gt; Pengaturan (Ctrl+,)</strong> untuk menyesuaikan beban perangkat keras Anda.</p>
            </div>

            <div class="trouble-item">
              <h3>3. Masalah Tampilan Cache Foto</h3>
              <p>Jika pratinjau foto tidak mencerminkan perubahan asli atau terasa tersendat, Anda dapat membersihkan cache dengan mengeklik menu <strong>Workspace &gt; Bersihkan Cache Gambar</strong>. Aplikasi akan memuat ulang thumbnail segar.</p>
            </div>
          {:else}
            <h2>🛠️ Troubleshooting & Support</h2>
            <p class="tab-subtitle">Solutions to common issues encountered in Keepix.</p>

            <div class="trouble-item">
              <h3>1. Video is Black or Only Plays Audio</h3>
              <p>This is usually due to the video file using H.265/HEVC encoding. Refer to the <strong>Video Compatibility</strong> tab to install the HEVC codec extension via the Microsoft Store.</p>
            </div>

            <div class="trouble-item">
              <h3>2. The UI is Lagging or Unresponsive</h3>
              <p>Keepix uses multi-threading to process RAW images and 4K videos. You can adjust the number of CPU threads used in <strong>Edit &gt; Settings (Ctrl+,)</strong> to better fit your computer's specifications.</p>
            </div>

            <div class="trouble-item">
              <h3>3. Thumbnail / Image Cache Mismatch</h3>
              <p>If thumbnails look incorrect or corrupted, clean the render cache by going to <strong>Workspace &gt; Clear Image Cache</strong>. The app will regenerate clean assets.</p>
            </div>
          {/if}
        </div>
      {/if}

    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(10px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 11000;
  }

  .modal-container {
    width: 820px;
    height: 520px;
    background: var(--surface-glass);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl), var(--shadow-glow);
    display: flex;
    overflow: hidden;
    animation: zoomIn 0.2s cubic-bezier(0.2, 0.8, 0.2, 1) forwards;
  }

  /* Sidebar styling */
  .modal-sidebar {
    width: 200px;
    background: rgba(20, 20, 23, 0.8);
    border-right: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    padding: var(--space-4) 0;
    flex-shrink: 0;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-4) var(--space-4) var(--space-4);
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    margin-bottom: var(--space-3);
  }

  .help-icon {
    color: var(--accent);
  }

  .sidebar-title {
    font-size: var(--text-xs);
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: white;
  }

  .sidebar-menu {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    padding: 0 var(--space-2);
  }

  .menu-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 11px;
    font-weight: 500;
    text-align: left;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .menu-btn:hover {
    background: rgba(255, 255, 255, 0.03);
    color: white;
  }

  .menu-btn.active {
    background: var(--accent);
    color: white;
    font-weight: 600;
  }

  .sidebar-footer {
    padding: 0 var(--space-4);
    font-size: 9px;
    font-family: var(--font-mono);
    color: var(--text-tertiary);
    opacity: 0.5;
  }

  /* Content area */
  .modal-content {
    flex: 1;
    padding: var(--space-6);
    overflow-y: auto;
    position: relative;
    display: flex;
    flex-direction: column;
  }

  .close-btn {
    position: absolute;
    top: var(--space-4);
    right: var(--space-4);
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: var(--text-md);
    transition: color var(--transition-fast);
  }

  .close-btn:hover {
    color: white;
  }

  .content-tab {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .content-tab h2 {
    font-size: var(--text-md);
    font-weight: 700;
    color: white;
    margin: 0;
  }

  .tab-subtitle {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    margin: 0 0 var(--space-2) 0;
  }

  /* Quick Start steps */
  .step-card {
    display: flex;
    gap: var(--space-4);
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.04);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
  }

  .step-num {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--accent-soft);
    border: 1px solid rgba(99, 102, 241, 0.3);
    color: var(--accent-light);
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: var(--text-xs);
    flex-shrink: 0;
  }

  .step-body h3 {
    font-size: var(--text-xs);
    font-weight: 600;
    color: white;
    margin: 0 0 4px 0;
  }

  .step-body p {
    font-size: 11px;
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.4;
  }

  .step-body ul {
    margin: var(--space-2) 0 0 0;
    padding-left: var(--space-4);
    font-size: 11px;
    color: var(--text-secondary);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 14px;
    height: 14px;
    padding: 0 3px;
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 700;
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 3px;
  }

  /* Video compatibility */
  .info-block {
    background: rgba(245, 158, 11, 0.08);
    border: 1px solid rgba(245, 158, 11, 0.2);
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-4);
  }

  .info-block h3 {
    font-size: var(--text-xs);
    font-weight: 600;
    color: #f59e0b;
    margin: 0 0 var(--space-1) 0;
  }

  .info-block p {
    font-size: 11px;
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.4;
  }

  .format-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-3);
  }

  .format-card {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.04);
    padding: var(--space-3);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .format-name {
    font-size: var(--text-xs);
    font-weight: 700;
    color: white;
  }

  .status-ok {
    font-size: 8px;
    font-weight: 700;
    text-transform: uppercase;
    color: var(--color-simpan);
    background: rgba(34, 197, 94, 0.15);
    padding: 2px 6px;
    border-radius: var(--radius-full);
    align-self: start;
    border: 1px solid rgba(34, 197, 94, 0.3);
  }

  .status-warn {
    font-size: 8px;
    font-weight: 700;
    text-transform: uppercase;
    color: #f59e0b;
    background: rgba(245, 158, 11, 0.15);
    padding: 2px 6px;
    border-radius: var(--radius-full);
    align-self: start;
    border: 1px solid rgba(245, 158, 11, 0.3);
  }

  .format-card p {
    font-size: 10px;
    color: var(--text-tertiary);
    margin: 0;
    line-height: 1.3;
  }

  .step-card.highlight {
    border-color: rgba(99, 102, 241, 0.3);
    background: rgba(99, 102, 241, 0.03);
  }

  /* Shortcuts Table */
  .shortcuts-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
    color: var(--text-secondary);
  }

  .shortcuts-table th {
    text-align: left;
    padding: var(--space-2) var(--space-3);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    color: white;
    font-weight: 600;
    font-size: var(--text-xs);
  }

  .shortcuts-table td {
    padding: var(--space-2) var(--space-3);
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }

  .shortcuts-table tr:hover td {
    background: rgba(255, 255, 255, 0.01);
    color: white;
  }

  /* Troubleshooting */
  .trouble-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding-bottom: var(--space-3);
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .trouble-item:last-child {
    border-bottom: none;
  }

  .trouble-item h3 {
    font-size: var(--text-xs);
    font-weight: 600;
    color: white;
    margin: 0;
  }

  .trouble-item p {
    font-size: 11px;
    color: var(--text-secondary);
    margin: 0;
    line-height: 1.4;
  }

  @keyframes zoomIn {
    from { opacity: 0; transform: scale(0.95); }
    to { opacity: 1; transform: scale(1); }
  }
</style>
