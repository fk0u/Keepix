import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export const showSettings = writable(false);
export const showAbout = writable(false);
export const showShortcuts = writable(false);
export const showExport = writable(false);
export const showHelp = writable(false);
export const editPanelCollapsed = writable(false);

export const activeTheme = writable('resolve-dark');

export async function initTheme() {
  try {
    const savedTheme = await invoke<string | null>('get_setting', { key: 'theme' });
    if (savedTheme) {
      activeTheme.set(savedTheme);
      applyThemeClass(savedTheme);
    } else {
      applyThemeClass('resolve-dark');
    }
  } catch (err) {
    console.warn('Failed to load theme setting:', err);
    applyThemeClass('resolve-dark');
  }
}

export async function setTheme(themeName: string) {
  activeTheme.set(themeName);
  applyThemeClass(themeName);
  try {
    await invoke('set_setting', { key: 'theme', value: themeName });
  } catch (err) {
    console.warn('Failed to save theme setting:', err);
  }
}

function applyThemeClass(themeName: string) {
  if (typeof document !== 'undefined') {
    const root = document.documentElement;
    root.classList.remove('theme-resolve-dark', 'theme-lightroom', 'theme-cyberpunk');
    root.classList.add(`theme-${themeName}`);
  }
}

export const showAiCull = writable(false);
