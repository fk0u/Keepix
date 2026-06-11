<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import {
    projects,
    currentProject,
    loadProjects,
    openFolderAndCreateProject,
    createProjectFromFolder,
    removeProject,
    setCurrentProject,
    scanProgress,
    isScanning,
    startScan,
  } from '$lib/stores/project';
  import { resetMediaStore, loadMediaItems, loadCategories, loadCategoryStats } from '$lib/stores/media';
  import { toast } from '$lib/stores/toast';
  import type { Project } from '$lib/types';

  let isDragging = $state(false);

  onMount(async () => {
    await loadProjects();
    await loadCategories();
  });

  async function handleOpenFolder() {
    const project = await openFolderAndCreateProject();
    if (project) {
      await openProject(project);
    }
  }

  async function openProject(project: Project) {
    setCurrentProject(project);
    resetMediaStore();

    // Start scanning if new project (0 items)
    if (project.total_items === 0) {
      await startScan(project.id, project.root_path);
    }

    // Load items and navigate
    await loadMediaItems(project.id);
    await loadCategoryStats(project.id);
    goto('/cull');
  }

  async function handleDeleteProject(e: MouseEvent, projectId: string) {
    e.stopPropagation();
    await removeProject(projectId);
    toast.info('Project removed');
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  function handleDragLeave() {
    isDragging = false;
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;

    const items = e.dataTransfer?.items;
    if (items && items.length > 0) {
      // Note: In Tauri, drag-and-drop of folders is handled differently
      // For now, use the folder picker
      toast.info('Use the "Open Folder" button to select a folder');
    }
  }

  function formatDate(dateStr: string): string {
    try {
      const d = new Date(dateStr);
      return d.toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: 'numeric',
      });
    } catch {
      return dateStr;
    }
  }
</script>

<div
  class="home"
  class:dragging={isDragging}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
  role="main"
>
  <!-- Hero section -->
  <div class="hero">
    <div class="logo">
      <div class="logo-icon">
        <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
          <rect x="4" y="8" width="40" height="32" rx="4" stroke="currentColor" stroke-width="2.5" fill="none"/>
          <circle cx="24" cy="24" r="8" stroke="currentColor" stroke-width="2" fill="none"/>
          <circle cx="24" cy="24" r="3" fill="currentColor"/>
          <circle cx="36" cy="14" r="2" fill="currentColor"/>
        </svg>
      </div>
      <h1 class="logo-text">Keepix</h1>
    </div>
    <p class="hero-subtitle">Fast, lightweight photo & video culling</p>

    <button class="btn btn-primary btn-lg open-btn" onclick={handleOpenFolder}>
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
        <line x1="12" y1="11" x2="12" y2="17"/>
        <line x1="9" y1="14" x2="15" y2="14"/>
      </svg>
      Open Folder
    </button>
  </div>

  <!-- Recent projects -->
  {#if $projects.length > 0}
    <div class="projects-section">
      <h2 class="section-title">Recent Projects</h2>
      <div class="projects-grid">
        {#each $projects as project (project.id)}
          <div
            class="project-card glass-card"
            role="button"
            tabindex="0"
            onclick={() => openProject(project)}
            onkeydown={(e) => e.key === 'Enter' && openProject(project)}
          >
            <div class="project-card-header">
              <div class="project-icon">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
                </svg>
              </div>
              <button
                class="btn btn-ghost btn-icon delete-btn"
                onclick={(e) => handleDeleteProject(e, project.id)}
                data-tooltip="Delete project"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18"/>
                  <line x1="6" y1="6" x2="18" y2="18"/>
                </svg>
              </button>
            </div>
            <h3 class="project-name truncate">{project.name}</h3>
            <p class="project-path truncate">{project.root_path}</p>
            <div class="project-meta">
              <span>{project.total_items} items</span>
              <span>{formatDate(project.last_opened)}</span>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Empty state -->
  {#if $projects.length === 0}
    <div class="empty-state">
      <p class="empty-hint">Select a folder containing your photos or videos to start culling</p>
      <div class="hotkeys-hint">
        <div class="hotkey-row">
          <kbd>1</kbd> <span>Buang (Trash)</span>
        </div>
        <div class="hotkey-row">
          <kbd>2</kbd> <span>Simpan (Best)</span>
        </div>
        <div class="hotkey-row">
          <kbd>3</kbd> <span>Draft (Keep)</span>
        </div>
        <div class="hotkey-row">
          <kbd>4</kbd> <span>Review</span>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .home {
    width: 100%;
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-10);
    padding: var(--space-8);
    position: relative;
    overflow-y: auto;
  }

  .home.dragging::after {
    content: 'Drop folder here';
    position: absolute;
    inset: var(--space-4);
    border: 2px dashed var(--accent);
    border-radius: var(--radius-xl);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--text-xl);
    color: var(--accent);
    background: var(--accent-soft);
    z-index: 10;
    pointer-events: none;
  }

  /* Hero */
  .hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-5);
    animation: slideUp 0.5s ease forwards;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: var(--space-4);
  }

  .logo-icon {
    color: var(--accent);
    opacity: 0.9;
  }

  .logo-text {
    font-size: var(--text-3xl);
    font-weight: 700;
    background: linear-gradient(135deg, #818cf8, #a78bfa, #c084fc);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    letter-spacing: -0.5px;
  }

  .hero-subtitle {
    color: var(--text-secondary);
    font-size: var(--text-md);
  }

  .open-btn {
    margin-top: var(--space-2);
    padding: var(--space-3) var(--space-8);
    font-size: var(--text-md);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-glow);
    transition: all var(--transition-base);
  }

  .open-btn:hover {
    box-shadow: 0 0 30px rgba(99, 102, 241, 0.35);
    transform: translateY(-1px);
  }

  /* Projects */
  .projects-section {
    width: 100%;
    max-width: 800px;
    animation: fadeIn 0.5s ease 0.15s both;
  }

  .section-title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 1px;
    margin-bottom: var(--space-4);
  }

  .projects-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: var(--space-3);
  }

  .project-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-4);
    cursor: pointer;
    transition: all var(--transition-base);
    text-align: left;
    font-family: var(--font-sans);
    width: 100%;
  }

  .project-card:hover {
    background: var(--surface-card-hover);
    border-color: var(--border-default);
    transform: translateY(-2px);
    box-shadow: var(--shadow-md);
  }

  .project-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .project-icon {
    color: var(--accent);
    opacity: 0.7;
  }

  .delete-btn {
    opacity: 0;
    transition: opacity var(--transition-fast);
    min-width: 24px;
    min-height: 24px;
    padding: 4px;
  }

  .project-card:hover .delete-btn {
    opacity: 0.5;
  }

  .delete-btn:hover {
    opacity: 1 !important;
    color: var(--color-buang);
  }

  .project-name {
    font-size: var(--text-md);
    font-weight: 600;
    color: var(--text-primary);
  }

  .project-path {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    font-family: var(--font-mono);
  }

  .project-meta {
    display: flex;
    justify-content: space-between;
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    margin-top: var(--space-1);
  }

  /* Empty state */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-6);
    animation: fadeIn 0.5s ease 0.3s both;
  }

  .empty-hint {
    color: var(--text-tertiary);
    font-size: var(--text-sm);
    text-align: center;
    max-width: 400px;
  }

  .hotkeys-hint {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-2) var(--space-6);
  }

  .hotkey-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    color: var(--text-tertiary);
  }

  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 24px;
    padding: 0 6px;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
  }
</style>
