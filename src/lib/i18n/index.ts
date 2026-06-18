// ============================================================================
// Keepix — i18n (Internationalization) Service
// Lightweight custom store for localization without heavy dependencies
// ============================================================================

import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Supported languages
export type Language = 'en' | 'id';

// Translation dictionaries
const translations = {
  en: {
    // General
    'app.name': 'Keepix by KOU',
    'app.tagline': 'Photo & Video Culling',
    'app.version': 'v4.0.1',
    'btn.new_workspace': 'New Workspace',
    'btn.open_folder': 'Open Folder',
    
    // Homepage Guide
    'guide.title': 'Getting Started',
    'guide.step1.title': 'Open a folder',
    'guide.step1.desc': 'Select a folder containing your photos or videos. Keepix will scan and generate thumbnails automatically.',
    'guide.step2.title': 'Categorize with hotkeys',
    'guide.step2.desc': 'Press 1 (Trash), 2 (Best), 3 (Draft), or 4 (Review) to quickly sort your photos.',
    'guide.step3.title': 'Rate & Label',
    'guide.step3.desc': 'Use 0-5 for star ratings and 6-9 for color labels. Toggle Auto-Advance with A to speed up workflow.',
    'guide.step4.title': 'Export your picks',
    'guide.step4.desc': 'Use the Export panel to copy, move, or list your curated selection to a destination folder.',
    
    // Quick Reference
    'ref.title': 'Quick Reference',
    'ref.trash': 'Trash',
    'ref.best': 'Best',
    'ref.draft': 'Draft',
    'ref.review': 'Review',
    'ref.toggle_view': 'Toggle View',
    'ref.auto_advance': 'Auto-Advance',
    
    // Homepage Main
    'home.welcome': 'Welcome back',
    'home.subtitle': 'Pick up where you left off, or start a new workspace.',
    'home.search': 'Search projects...',
    'home.stats.projects': 'Projects',
    'home.stats.items': 'Total Items',
    'home.recent': 'Recent Workspaces',
    'home.empty.title': 'Start Your First Workspace',
    'home.empty.desc': 'Select a folder containing your photos or videos to begin culling. Keepix will scan your files, generate thumbnails, and set up your workspace automatically.',
    'home.feat.fast': 'Lightning-fast culling with keyboard shortcuts',
    'home.feat.focus': 'Focus peaking & exposure diagnostics',
    'home.feat.hist': 'Real-time RGB histogram',
    'home.feat.burst': 'Smart burst grouping & compare mode',
    'home.no_match': 'No projects matching',
    
    // Edit Panel
    'edit.title': 'Edit',
    'edit.basic': 'Basic',
    'edit.color': 'Color',
    'edit.presets': 'Presets',
    'edit.exposure': 'Exposure',
    'edit.contrast': 'Contrast',
    'edit.highlights': 'Highlights',
    'edit.shadows': 'Shadows',
    'edit.temperature': 'Temperature',
    'edit.tint': 'Tint',
    'edit.saturation': 'Saturation',
    'edit.vibrance': 'Vibrance',
    'edit.reset': 'Reset All',
    'edit.preset.none': 'None',
    'edit.preset.cinematic': 'Cinematic',
    'edit.preset.vintage': 'Vintage',
    'edit.preset.bw': 'B&W High Contrast',
    'edit.preset.punchy': 'Punchy Color',

    // Settings
    'settings.title': 'Settings',
    'settings.language': 'Language',
    'settings.language.en': 'English',
    'settings.language.id': 'Bahasa Indonesia',
    'settings.performance': 'Performance',
    'settings.cache_limit': 'Image Cache Limit',
    'settings.cache.info': 'Higher values use more memory but make scrolling smoother.',
    'settings.save': 'Save Changes',
    'settings.cancel': 'Cancel',

    // About
    'about.title': 'About Keepix',
    'about.desc': 'A blazing fast, keyboard-first photo culling and management tool.',
    'about.close': 'Close'
  },
  id: {
    // General
    'app.name': 'Keepix by KOU',
    'app.tagline': 'Seleksi Foto & Video',
    'app.version': 'v4.0.1',
    'btn.new_workspace': 'Workspace Baru',
    'btn.open_folder': 'Buka Folder',
    
    // Homepage Guide
    'guide.title': 'Panduan Memulai',
    'guide.step1.title': 'Buka sebuah folder',
    'guide.step1.desc': 'Pilih folder yang berisi foto atau video Anda. Keepix akan memindai dan membuat thumbnail secara otomatis.',
    'guide.step2.title': 'Kategorikan dengan tombol',
    'guide.step2.desc': 'Tekan 1 (Buang), 2 (Simpan), 3 (Draft), atau 4 (Review) untuk memilah foto dengan cepat.',
    'guide.step3.title': 'Beri Rating & Label',
    'guide.step3.desc': 'Gunakan 0-5 untuk rating bintang dan 6-9 untuk label warna. Tekan A untuk Auto-Advance agar lebih cepat.',
    'guide.step4.title': 'Ekspor pilihan Anda',
    'guide.step4.desc': 'Gunakan panel Ekspor untuk menyalin, memindahkan, atau mendaftar foto yang sudah disortir ke folder tujuan.',
    
    // Quick Reference
    'ref.title': 'Referensi Cepat',
    'ref.trash': 'Buang',
    'ref.best': 'Simpan (Terbaik)',
    'ref.draft': 'Draft',
    'ref.review': 'Review',
    'ref.toggle_view': 'Ubah Tampilan',
    'ref.auto_advance': 'Maju Otomatis',
    
    // Homepage Main
    'home.welcome': 'Selamat datang kembali',
    'home.subtitle': 'Lanjutkan pekerjaan Anda, atau mulai workspace baru.',
    'home.search': 'Cari project...',
    'home.stats.projects': 'Project',
    'home.stats.items': 'Total File',
    'home.recent': 'Workspace Terakhir',
    'home.empty.title': 'Mulai Workspace Pertama Anda',
    'home.empty.desc': 'Pilih folder berisi foto atau video untuk mulai memilah. Keepix akan memindai file, membuat thumbnail, dan menyiapkan workspace Anda secara otomatis.',
    'home.feat.fast': 'Pemilahan super cepat dengan keyboard',
    'home.feat.focus': 'Fokus peaking & deteksi pencahayaan',
    'home.feat.hist': 'Histogram RGB real-time',
    'home.feat.burst': 'Grup burst pintar & mode banding',
    'home.no_match': 'Tidak ada project untuk',
    
    // Edit Panel
    'edit.title': 'Edit',
    'edit.basic': 'Dasar',
    'edit.color': 'Warna',
    'edit.presets': 'Preset',
    'edit.exposure': 'Pencahayaan',
    'edit.contrast': 'Kontras',
    'edit.highlights': 'Sorotan (Highlights)',
    'edit.shadows': 'Bayangan (Shadows)',
    'edit.temperature': 'Suhu Warna',
    'edit.tint': 'Corak Warna',
    'edit.saturation': 'Saturasi',
    'edit.vibrance': 'Keceriaan Warna',
    'edit.reset': 'Reset Semua',
    'edit.preset.none': 'Tanpa Preset',
    'edit.preset.cinematic': 'Sinematik',
    'edit.preset.vintage': 'Klasik (Vintage)',
    'edit.preset.bw': 'Hitam Putih Kontras Tinggi',
    'edit.preset.punchy': 'Warna Kuat',

    // Settings
    'settings.title': 'Pengaturan',
    'settings.language': 'Bahasa',
    'settings.language.en': 'English',
    'settings.language.id': 'Bahasa Indonesia',
    'settings.performance': 'Performa',
    'settings.cache_limit': 'Batas Cache Gambar',
    'settings.cache.info': 'Nilai tinggi memakan lebih banyak RAM tapi membuat scroll lebih mulus.',
    'settings.save': 'Simpan Perubahan',
    'settings.cancel': 'Batal',

    // About
    'about.title': 'Tentang Keepix',
    'about.desc': 'Aplikasi pengelolaan dan pemilihan foto super cepat berbasis keyboard.',
    'about.close': 'Tutup'
  }
};

type DictPath = keyof typeof translations['en'];

// The active language store
export const locale = writable<Language>('en');

// Initialize language from backend settings
export async function initLanguage() {
  try {
    const savedLang = await invoke<string | null>('get_setting', { key: 'language' });
    if (savedLang === 'id' || savedLang === 'en') {
      locale.set(savedLang as Language);
    }
  } catch (err) {
    console.warn('Failed to load language setting:', err);
  }
}

// Update language and save to backend
export async function setLanguage(lang: Language) {
  locale.set(lang);
  try {
    await invoke('set_setting', { key: 'language', value: lang });
  } catch (err) {
    console.warn('Failed to save language setting:', err);
  }
}

// The translation store function `$t('key')`
export const t = derived(locale, ($locale) => {
  return (key: string, variables?: Record<string, string>): string => {
    const dict = translations[$locale];
    let text = (dict as any)[key] || (translations['en'] as any)[key] || key;
    
    if (variables) {
      for (const [k, v] of Object.entries(variables)) {
        text = text.replace(new RegExp(`{${k}}`, 'g'), v);
      }
    }
    
    return text;
  };
});
