<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { showSettings, showAbout, showShortcuts, showExport, showHelp } from '$lib/stores/ui';
  import { currentProject } from '$lib/stores/project';
  import { get } from 'svelte/store';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { t, locale } from '$lib/i18n';

  let activeMenu = $state<string | null>(null);
  let menuContainer = $state<HTMLDivElement>();

  let cpuThreads = $state(8);
  let gpuRenderer = $state('Detecting GPU...');

  function toggleMenu(menuName: string) {
    if (activeMenu === menuName) {
      activeMenu = null;
    } else {
      activeMenu = menuName;
    }
  }

  function handleMouseOver(menuName: string) {
    if (activeMenu !== null) {
      activeMenu = menuName;
    }
  }

  function closeMenu() {
    activeMenu = null;
  }

  function triggerAction(actionName: string) {
    closeMenu();
    
    // Handle global dialogs directly
    if (actionName === 'open-settings') {
      showSettings.set(true);
      return;
    }
    if (actionName === 'open-about') {
      showAbout.set(true);
      return;
    }
    if (actionName === 'open-shortcuts') {
      showShortcuts.set(true);
      return;
    }
    if (actionName === 'open-export') {
      showExport.set(true);
      return;
    }
    if (actionName === 'open-help') {
      showHelp.set(true);
      return;
    }
    if (actionName === 'go-home') {
      goto('/');
      return;
    }
    if (actionName === 'exit-app') {
      try {
        getCurrentWindow().close();
      } catch (err) {
        console.warn('Cannot close tauri window:', err);
      }
      return;
    }

    // Otherwise dispatch a window custom event for the current page
    window.dispatchEvent(new CustomEvent('keepix-action', { detail: actionName }));
  }

  onMount(() => {
    // Detect hardware concurrency
    if (typeof navigator !== 'undefined' && navigator.hardwareConcurrency) {
      cpuThreads = navigator.hardwareConcurrency;
    }
    
    // Detect GPU via WebGL
    try {
      const canvas = document.createElement('canvas');
      const gl = (canvas.getContext('webgl2') || canvas.getContext('webgl') || canvas.getContext('experimental-webgl')) as WebGLRenderingContext | null;
      if (gl) {
        const ext = gl.getExtension('WEBGL_debug_renderer_info');
        if (ext) {
          const rawRenderer = gl.getParameter(ext.UNMASKED_RENDERER_WEBGL);
          // Clean up common prefixes to keep it clean (e.g. "ANGLE (NVIDIA GeForce RTX 4070 Direct3D11)")
          gpuRenderer = rawRenderer
            .replace(/^ANGLE \((.*)\)$/, '$1')
            .split(' vs_')[0]
            .split(' Direct3D')[0];
        } else {
          gpuRenderer = 'Hardware Accelerated';
        }
      } else {
        gpuRenderer = 'Software Emulation';
      }
    } catch {
      gpuRenderer = 'GPU Accel';
    }

    function handleOutsideClick(e: MouseEvent) {
      if (menuContainer && !menuContainer.contains(e.target as Node)) {
        closeMenu();
      }
    }
    window.addEventListener('click', handleOutsideClick);
    return () => {
      window.removeEventListener('click', handleOutsideClick);
    };
  });
</script>

<div class="menu-bar" bind:this={menuContainer}>
  <!-- App Branding Area -->
  <div class="menu-bar-brand" onclick={() => triggerAction('go-home')} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && triggerAction('go-home')}>
    <div class="brand-logo-mini">
      <svg width="14" height="14" viewBox="0 0 48 48" fill="none">
        <rect x="4" y="8" width="40" height="32" rx="4" stroke="currentColor" stroke-width="3" fill="none"/>
        <circle cx="24" cy="24" r="8" stroke="currentColor" stroke-width="2.5" fill="none"/>
        <circle cx="24" cy="24" r="3" fill="currentColor"/>
      </svg>
    </div>
    <span class="brand-title">Keepix <span class="brand-by">by KOU</span></span>
    <span class="version-pill">v4.1.0</span>
  </div>

  <!-- Menu Triggers & Dropdowns -->
  <div class="menu-items">
    
    <!-- FILE MENU -->
    <div class="menu-item-container">
      <button 
        class="menu-trigger-btn" 
        class:active={activeMenu === 'File'} 
        onclick={() => toggleMenu('File')} 
        onmouseover={() => handleMouseOver('File')}
      >
        {$t('menu.file')}
      </button>
      {#if activeMenu === 'File'}
        <div class="menu-dropdown">
          <button class="dropdown-item" onclick={() => triggerAction('new-workspace')}>
            <span class="item-text">{$t('menu.file.new')}</span>
            <span class="item-shortcut">Ctrl+N</span>
          </button>
          <button class="dropdown-item" onclick={() => triggerAction('rescan-project')} disabled={!$currentProject}>
            <span class="item-text">{$t('menu.file.rescan')}</span>
            <span class="item-shortcut">Ctrl+R</span>
          </button>
          <button class="dropdown-item" onclick={() => triggerAction('open-export')} disabled={!$currentProject}>
            <span class="item-text">{$t('menu.file.export')}</span>
            <span class="item-shortcut">Ctrl+E</span>
          </button>
          <div class="dropdown-divider"></div>
          <button class="dropdown-item danger" onclick={() => triggerAction('exit-app')}>
            <span class="item-text">{$t('menu.file.exit')}</span>
            <span class="item-shortcut">Alt+F4</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- EDIT MENU -->
    <div class="menu-item-container">
      <button 
        class="menu-trigger-btn" 
        class:active={activeMenu === 'Edit'} 
        onclick={() => toggleMenu('Edit')} 
        onmouseover={() => handleMouseOver('Edit')}
      >
        {$t('menu.edit')}
      </button>
      {#if activeMenu === 'Edit'}
        <div class="menu-dropdown">
          <button class="dropdown-item" onclick={() => triggerAction('undo')} disabled={!$currentProject}>
            <span class="item-text">{$t('menu.edit.undo')}</span>
            <span class="item-shortcut">Ctrl+Z</span>
          </button>
          <button class="dropdown-item" onclick={() => triggerAction('reset-adjustments')} disabled={!$currentProject}>
            <span class="item-text">{$t('menu.edit.reset')}</span>
            <span class="item-shortcut">Alt+R</span>
          </button>
          <div class="dropdown-divider"></div>
          <button class="dropdown-item" onclick={() => triggerAction('open-settings')}>
            <span class="item-text">{$t('menu.edit.settings')}</span>
            <span class="item-shortcut">Ctrl+,</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- VIEW MENU -->
    <div class="menu-item-container">
      <button 
        class="menu-trigger-btn" 
        class:active={activeMenu === 'View'} 
        onclick={() => toggleMenu('View')} 
        onmouseover={() => handleMouseOver('View')}
      >
        {$t('menu.view')}
      </button>
      {#if activeMenu === 'View'}
        <div class="menu-dropdown">
          <button class="dropdown-item" onclick={() => triggerAction('view-grid')}>
            <span class="item-text">{$t('menu.view.grid')}</span>
            <span class="item-shortcut">Space</span>
          </button>
          <button class="dropdown-item" onclick={() => triggerAction('view-preview')}>
            <span class="item-text">{$t('menu.view.preview')}</span>
            <span class="item-shortcut">Space</span>
          </button>
          <div class="dropdown-divider"></div>
          <button class="dropdown-item" onclick={() => triggerAction('compare-single')}>
            <span class="item-text">{$t('menu.view.compare_single')}</span>
            <span class="item-shortcut">C</span>
          </button>
          <button class="dropdown-item" onclick={() => triggerAction('compare-2up')}>
            <span class="item-text">{$t('menu.view.compare_2up')}</span>
            <span class="item-shortcut">C</span>
          </button>
          <button class="dropdown-item" onclick={() => triggerAction('compare-4up')}>
            <span class="item-text">{$t('menu.view.compare_4up')}</span>
            <span class="item-shortcut">C</span>
          </button>
          <div class="dropdown-divider"></div>
          <button class="dropdown-item" onclick={() => triggerAction('toggle-metadata')}>
            <span class="item-text">{$t('menu.view.info')}</span>
            <span class="item-shortcut">I</span>
          </button>
          <button class="dropdown-item" onclick={() => triggerAction('toggle-histogram')}>
            <span class="item-text">{$t('menu.view.histogram')}</span>
            <span class="item-shortcut">H</span>
          </button>
          <button class="dropdown-item" onclick={() => triggerAction('toggle-overlay')}>
            <span class="item-text">{$t('menu.view.overlays')}</span>
            <span class="item-shortcut">O</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- WORKSPACE MENU -->
    <div class="menu-item-container">
      <button 
        class="menu-trigger-btn" 
        class:active={activeMenu === 'Workspace'} 
        onclick={() => toggleMenu('Workspace')} 
        onmouseover={() => handleMouseOver('Workspace')}
      >
        {$t('menu.workspace')}
      </button>
      {#if activeMenu === 'Workspace'}
        <div class="menu-dropdown">
          <button class="dropdown-item" onclick={() => triggerAction('go-home')}>
            <span class="item-text">{$t('menu.workspace.home')}</span>
            <span class="item-shortcut">Esc</span>
          </button>
          <button class="dropdown-item" onclick={() => triggerAction('clear-image-cache')}>
            <span class="item-text">{$t('menu.workspace.clear_cache')}</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- HELP MENU -->
    <div class="menu-item-container">
      <button 
        class="menu-trigger-btn" 
        class:active={activeMenu === 'Help'} 
        onclick={() => toggleMenu('Help')} 
        onmouseover={() => handleMouseOver('Help')}
      >
        {$t('menu.help')}
      </button>
      {#if activeMenu === 'Help'}
        <div class="menu-dropdown">
          <button class="dropdown-item" onclick={() => triggerAction('open-shortcuts')}>
            <span class="item-text">{$t('menu.help.shortcuts')}</span>
            <span class="item-shortcut">?</span>
          </button>
          <button class="dropdown-item" onclick={() => triggerAction('open-help')}>
            <span class="item-text">{$t('menu.help.docs')}</span>
            <span class="item-shortcut">F1</span>
          </button>
          <div class="dropdown-divider"></div>
          <button class="dropdown-item" onclick={() => triggerAction('open-about')}>
            <span class="item-text">{$t('menu.help.about')}</span>
          </button>
        </div>
      {/if}
    </div>

  </div>

  <!-- Active Workspace status indicator -->
  <div class="menu-bar-status">
    <div class="hardware-indicators">
      <span class="hw-badge cpu-badge">
        <span class="hw-dot green"></span>
        {$t('menu.status.cpu', {cores: cpuThreads.toString()})}
      </span>
      <span class="hw-badge gpu-badge" title={gpuRenderer}>
        <span class="hw-dot green"></span>
        {$t('menu.status.gpu', {gpu: gpuRenderer.length > 25 ? gpuRenderer.substring(0, 22) + '...' : gpuRenderer})}
      </span>
    </div>
    <div class="divider-v"></div>
    {#if $currentProject}
      <span class="status-project-indicator">
        {@html $t('menu.status.active_workspace', {name: `<strong class="project-name-text">${$currentProject.name}</strong>`})}
      </span>
    {:else}
      <span class="status-project-indicator idle">
        {$t('menu.status.no_project')}
      </span>
    {/if}
  </div>
</div>

<style>
  .menu-bar {
    height: 32px;
    background: #121214;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    display: flex;
    align-items: center;
    padding: 0 var(--space-4);
    user-select: none;
    z-index: 10000;
    flex-shrink: 0;
  }

  .menu-bar-brand {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-right: var(--space-6);
    cursor: pointer;
    padding: 2px 4px;
    border-radius: var(--radius-sm);
  }

  .menu-bar-brand:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .brand-logo-mini {
    color: var(--accent);
    display: flex;
    align-items: center;
  }

  .brand-title {
    font-size: var(--text-xs);
    font-weight: 700;
    color: white;
    letter-spacing: -0.2px;
  }

  .brand-by {
    color: var(--accent-light);
    font-weight: 500;
    font-size: 10px;
    opacity: 0.95;
    margin-left: 1px;
  }

  .version-pill {
    background: var(--accent-soft);
    color: white;
    font-family: var(--font-mono);
    font-size: 8px;
    font-weight: 600;
    padding: 1px 4px;
    border-radius: var(--radius-full);
    border: 1px solid rgba(99, 102, 241, 0.2);
    margin-left: 4px;
  }

  .menu-items {
    display: flex;
    align-items: center;
    gap: 2px;
    height: 100%;
  }

  .menu-item-container {
    position: relative;
    height: 100%;
    display: flex;
    align-items: center;
  }

  .menu-trigger-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 11px;
    font-weight: 500;
    padding: 0 var(--space-3);
    height: 24px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    transition: all 0.1s;
  }

  .menu-trigger-btn:hover, .menu-trigger-btn.active {
    background: rgba(255, 255, 255, 0.05);
    color: white;
  }

  .menu-dropdown {
    position: absolute;
    top: 28px;
    left: 0;
    background: #1c1c1f;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: var(--radius-md);
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
    padding: var(--space-1) 0;
    min-width: 220px;
    z-index: 10001;
    display: flex;
    flex-direction: column;
  }

  .dropdown-item {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 11px;
    padding: 6px var(--space-4);
    text-align: left;
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
  }

  .dropdown-item:hover:not(:disabled) {
    background: var(--accent);
    color: white;
  }

  .dropdown-item:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .dropdown-item.danger:hover {
    background: var(--color-buang);
  }

  .item-shortcut {
    font-family: var(--font-mono);
    font-size: 9px;
    opacity: 0.5;
    padding-left: var(--space-4);
  }

  .dropdown-divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.05);
    margin: var(--space-1) 0;
  }

  .menu-bar-status {
    margin-left: auto;
    font-size: 10px;
    color: var(--text-tertiary);
    font-family: var(--font-sans);
    display: flex;
    align-items: center;
    gap: var(--space-4);
  }

  .hardware-indicators {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .hw-badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.04);
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
    font-size: 9px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }

  .hw-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    display: inline-block;
  }

  .hw-dot.green {
    background: #10b981;
    box-shadow: 0 0 6px rgba(16, 185, 129, 0.7);
  }

  .divider-v {
    width: 1px;
    height: 14px;
    background: rgba(255, 255, 255, 0.08);
  }

  .status-project-indicator {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    background: rgba(255, 255, 255, 0.02);
    padding: 2px 8px;
    border-radius: var(--radius-md);
    border: 1px solid rgba(255, 255, 255, 0.03);
  }

  .status-project-indicator.idle {
    opacity: 0.5;
  }

  .project-name-text {
    color: var(--accent-light);
  }
</style>
