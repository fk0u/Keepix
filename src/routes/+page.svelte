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
  import { t, locale } from '$lib/i18n';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import AboutModal from '$lib/components/AboutModal.svelte';
  import { showSettings, showAbout } from '$lib/stores/ui';

  let isDragging = $state(false);
  let searchQuery = $state('');
  let expandedGuide = $state<number | null>(null);

  // Filtered projects based on search
  let filteredProjects = $derived(
    searchQuery.trim()
      ? $projects.filter(p =>
          p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          p.root_path.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : $projects
  );

  // Computed stats
  let totalPhotos = $derived($projects.reduce((sum, p) => sum + p.total_items, 0));
  let totalProjects = $derived($projects.length);

  onMount(() => {
    loadProjects();
    loadCategories();

    function handleAction(e: Event) {
      const detail = (e as CustomEvent).detail;
      if (detail === 'new-workspace' || detail === 'trigger-folder-picker') {
        handleOpenFolder();
      } else if (detail === 'clear-image-cache') {
        import('$lib/services/image-cache').then(({ clearImageCache }) => {
          clearImageCache().then(() => toast.success($t('home.toast.cache_cleared')));
        });
      }
    }
    window.addEventListener('keepix-action', handleAction);
    return () => window.removeEventListener('keepix-action', handleAction);
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
    toast.info($t('home.toast.project_removed'));
  }

  async function handleRescan(e: MouseEvent, project: Project) {
    e.stopPropagation();
    setCurrentProject(project);
    resetMediaStore();
    await startScan(project.id, project.root_path);
    await loadMediaItems(project.id);
    await loadCategoryStats(project.id);
    goto('/cull');
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
    toast.info($t('home.toast.use_new_workspace'));
  }

  function formatDate(dateStr: string, activeLocale: string): string {
    try {
      const d = new Date(dateStr);
      const now = new Date();
      const diffMs = now.getTime() - d.getTime();
      const diffMins = Math.floor(diffMs / 60000);
      const diffHours = Math.floor(diffMs / 3600000);
      const diffDays = Math.floor(diffMs / 86400000);

      if (diffMins < 1) return $t('time.just_now');
      if (diffMins < 60) return $t('time.m_ago', { val: diffMins.toString() });
      if (diffHours < 24) return $t('time.h_ago', { val: diffHours.toString() });
      if (diffDays < 7) return $t('time.d_ago', { val: diffDays.toString() });

      return d.toLocaleDateString(activeLocale === 'id' ? 'id-ID' : 'en-US', {
        month: 'short',
        day: 'numeric',
        year: d.getFullYear() !== now.getFullYear() ? 'numeric' : undefined,
      });
    } catch {
      return dateStr;
    }
  }

  function toggleGuide(step: number) {
    expandedGuide = expandedGuide === step ? null : step;
  }

  // Guide steps derived from i18n to ensure reactivity on language change
  let guideSteps = $derived([
    {
      title: $t('guide.step1.title'),
      desc: $t('guide.step1.desc'),
      icon: '📂',
    },
    {
      title: $t('guide.step2.title'),
      desc: $t('guide.step2.desc'),
      icon: '⌨️',
    },
    {
      title: $t('guide.step3.title'),
      desc: $t('guide.step3.desc'),
      icon: '⭐',
    },
    {
      title: $t('guide.step4.title'),
      desc: $t('guide.step4.desc'),
      icon: '📤',
    },
  ]);

  let loadedProjectImages = $state<Record<string, string>>({});

  async function loadProjectThumbnail(path: string) {
    if (loadedProjectImages[path]) return;
    const { getImageDataUri } = await import('$lib/services/image-cache');
    const dataUri = await getImageDataUri(path);
    if (dataUri) {
      loadedProjectImages = { ...loadedProjectImages, [path]: dataUri };
    }
  }

  function lazyLoadProjectAction(node: HTMLElement, path: string | null) {
    if (!path) return;

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            loadProjectThumbnail(path);
            observer.unobserve(node);
          }
        }
      },
      { rootMargin: '200px' }
    );

    observer.observe(node);

    return {
      destroy() {
        observer.unobserve(node);
        observer.disconnect();
      },
    };
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
  <!-- Animated background -->
  <div class="home-bg"></div>

  <!-- LEFT SIDEBAR -->
  <aside class="home-sidebar">
    <!-- Branding -->
    <div class="home-sidebar-brand">
      <div class="brand-logo">
        <svg width="32" height="32" viewBox="0 0 48 48" fill="none">
          <rect x="4" y="8" width="40" height="32" rx="4" stroke="currentColor" stroke-width="2.5" fill="none"/>
          <circle cx="24" cy="24" r="8" stroke="currentColor" stroke-width="2" fill="none"/>
          <circle cx="24" cy="24" r="3" fill="currentColor"/>
          <circle cx="36" cy="14" r="2" fill="currentColor"/>
        </svg>
      </div>
      <div>
        <h1 class="brand-name">{$t('app.name')}</h1>
        <span class="brand-tagline">{$t('app.tagline')}</span>
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="sidebar-section">
      <button class="sidebar-action-btn primary" onclick={handleOpenFolder}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 5v14M5 12h14"/>
        </svg>
        {$t('btn.new_workspace')}
      </button>
    </div>

    <!-- Getting Started Guide -->
    <div class="sidebar-section">
      <div class="sidebar-section-title">{$t('guide.title')}</div>
      {#each guideSteps as step, idx}
        <button class="guide-step" class:expanded={expandedGuide === idx} onclick={() => toggleGuide(idx)}>
          <div class="guide-step-header">
            <span class="guide-step-icon">{step.icon}</span>
            <span class="guide-step-num">{idx + 1}.</span>
            <span class="guide-step-title">{step.title}</span>
            <svg class="guide-chevron" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
          </div>
          {#if expandedGuide === idx}
            <p class="guide-step-desc">{step.desc}</p>
          {/if}
        </button>
      {/each}
    </div>

    <!-- Keyboard Reference -->
    <div class="sidebar-section">
      <div class="sidebar-section-title">{$t('ref.title')}</div>
      <div class="shortcut-grid">
        <div class="shortcut-row"><kbd>1</kbd><span>{$t('ref.trash')}</span></div>
        <div class="shortcut-row"><kbd>2</kbd><span>{$t('ref.best')}</span></div>
        <div class="shortcut-row"><kbd>3</kbd><span>{$t('ref.draft')}</span></div>
        <div class="shortcut-row"><kbd>4</kbd><span>{$t('ref.review')}</span></div>
        <div class="shortcut-row"><kbd>Space</kbd><span>{$t('ref.toggle_view')}</span></div>
        <div class="shortcut-row"><kbd>A</kbd><span>{$t('ref.auto_advance')}</span></div>
      </div>
    </div>

    <!-- Sidebar Footer -->
    <div class="sidebar-footer">
      <button class="footer-icon-btn" onclick={() => showSettings.set(true)} data-tooltip={$t('settings.preferences')}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
        </svg>
      </button>
      <button class="footer-icon-btn" onclick={() => showAbout.set(true)} data-tooltip={$t('about.title')}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="12" y1="16" x2="12" y2="12"></line>
          <line x1="12" y1="8" x2="12.01" y2="8"></line>
        </svg>
      </button>
      <span class="version-tag">{$t('app.version')}</span>
    </div>
  </aside>

  <!-- MAIN CONTENT -->
  <div class="home-main">
    <!-- Header -->
    <header class="home-header">
      <div class="header-welcome">
        <h2 class="welcome-title">{$t('home.welcome')}</h2>
        <p class="welcome-subtitle">{$t('home.subtitle')}</p>
      </div>
      {#if $projects.length > 0}
        <div class="header-search">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"/>
            <line x1="21" y1="21" x2="16.65" y2="16.65"/>
          </svg>
          <input
            type="text"
            placeholder={$t('home.search')}
            bind:value={searchQuery}
            class="search-input"
          />
        </div>
      {/if}
    </header>

    <!-- Stats Bar -->
    {#if $projects.length > 0}
      <div class="stats-bar">
        <div class="stat-chip">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
          </svg>
          <span class="stat-value">{totalProjects}</span>
          <span class="stat-label">{$t('home.stats.projects')}</span>
        </div>
        <div class="stat-chip">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2"/>
            <circle cx="8.5" cy="8.5" r="1.5"/>
            <polyline points="21 15 16 10 5 21"/>
          </svg>
          <span class="stat-value">{totalPhotos.toLocaleString()}</span>
          <span class="stat-label">{$t('home.stats.items')}</span>
        </div>
      </div>
    {/if}

    <!-- Projects Content -->
    <div class="home-content">
      {#if filteredProjects.length > 0}
        <div class="content-section-title">{$t('home.recent')}</div>
        <div class="projects-grid">
          {#each filteredProjects as project, idx (project.id)}
            <div
              class="project-card glass-card"
              role="button"
              tabindex="0"
              onclick={() => openProject(project)}
              onkeydown={(e) => e.key === 'Enter' && openProject(project)}
              style="animation-delay: {idx * 50}ms"
            >
              <!-- Cover area with gradient or thumbnail -->
              <div class="project-cover" use:lazyLoadProjectAction={project.thumbnail_path}>
                {#if project.thumbnail_path && loadedProjectImages[project.thumbnail_path]}
                  <img
                    src={loadedProjectImages[project.thumbnail_path]}
                    alt={project.name}
                    class="project-thumb-img"
                    draggable="false"
                  />
                {:else}
                  <div class="cover-gradient"></div>
                  <div class="cover-icon">
                    <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.4">
                      <rect x="3" y="3" width="18" height="18" rx="2"/>
                      <circle cx="8.5" cy="8.5" r="1.5"/>
                      <polyline points="21 15 16 10 5 21"/>
                    </svg>
                  </div>
                {/if}
                <div class="project-item-count">
                  <span>{project.total_items}</span>
                  <span class="count-label">{$t('home.project.items')}</span>
                </div>
              </div>

              <!-- Card body -->
              <div class="project-body">
                <div class="project-card-header">
                  <h3 class="project-name truncate">{project.name}</h3>
                  <div class="project-actions">
                    <button
                      class="action-btn"
                      onclick={(e) => handleRescan(e, project)}
                      data-tooltip={$t('home.project.rescan')}
                    >
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="23 4 23 10 17 10"/>
                        <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
                      </svg>
                    </button>
                    <button
                      class="action-btn danger"
                      onclick={(e) => handleDeleteProject(e, project.id)}
                      data-tooltip={$t('home.project.remove')}
                    >
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <line x1="18" y1="6" x2="6" y2="18"/>
                        <line x1="6" y1="6" x2="18" y2="18"/>
                      </svg>
                    </button>
                  </div>
                </div>
                <p class="project-path truncate">{project.root_path}</p>
                <div class="project-meta">
                  <span class="meta-time">{formatDate(project.last_opened, $locale)}</span>
                </div>
              </div>
            </div>
          {/each}
        </div>
      {:else if $projects.length > 0 && searchQuery}
        <div class="empty-search">
          <p>{$t('home.no_match')} "<strong>{searchQuery}</strong>"</p>
        </div>
      {:else}
        <!-- Empty state / Onboarding -->
        <div class="onboarding-card glass-card">
          <div class="onboarding-icon">
            <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="3" width="18" height="18" rx="2"/>
              <circle cx="8.5" cy="8.5" r="1.5"/>
              <polyline points="21 15 16 10 5 21"/>
            </svg>
          </div>
          <h3 class="onboarding-title">{$t('home.empty.title')}</h3>
          <p class="onboarding-desc">
            {$t('home.empty.desc')}
          </p>
          <button class="btn btn-primary btn-lg onboarding-cta" onclick={handleOpenFolder}>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
              <line x1="12" y1="11" x2="12" y2="17"/>
              <line x1="9" y1="14" x2="15" y2="14"/>
            </svg>
            {$t('btn.open_folder')}
          </button>

          <div class="onboarding-features">
            <div class="feature-item">
              <span class="feature-icon">⚡</span>
              <span>{$t('home.feat.fast')}</span>
            </div>
            <div class="feature-item">
              <span class="feature-icon">🔍</span>
              <span>{$t('home.feat.focus')}</span>
            </div>
            <div class="feature-item">
              <span class="feature-icon">📊</span>
              <span>{$t('home.feat.hist')}</span>
            </div>
            <div class="feature-item">
              <span class="feature-icon">📸</span>
              <span>{$t('home.feat.burst')}</span>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<SettingsModal show={$showSettings} onClose={() => showSettings.set(false)} />
<AboutModal show={$showAbout} onClose={() => showAbout.set(false)} />

<style>
  .home {
    width: 100%;
    height: 100vh;
    display: flex;
    position: relative;
    overflow: hidden;
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
    z-index: 100;
    pointer-events: none;
  }

  /* ===== Animated Background ===== */
  .home-bg {
    position: absolute;
    inset: 0;
    z-index: 0;
    background:
      radial-gradient(ellipse at 10% 20%, rgba(99, 102, 241, 0.08) 0%, transparent 50%),
      radial-gradient(ellipse at 80% 80%, rgba(168, 85, 247, 0.06) 0%, transparent 50%),
      radial-gradient(ellipse at 50% 0%, rgba(129, 140, 248, 0.05) 0%, transparent 40%);
    animation: bgShift 20s ease-in-out infinite alternate;
  }

  @keyframes bgShift {
    0% { opacity: 0.8; transform: scale(1); }
    50% { opacity: 1; transform: scale(1.05); }
    100% { opacity: 0.8; transform: scale(1); }
  }

  /* ===== LEFT SIDEBAR ===== */
  .home-sidebar {
    width: 280px;
    height: 100%;
    background: rgba(17, 17, 20, 0.6);
    backdrop-filter: blur(24px);
    -webkit-backdrop-filter: blur(24px);
    border-right: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    z-index: 2;
    overflow-y: auto;
  }

  .home-sidebar-brand {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-5) var(--space-4);
    border-bottom: 1px solid var(--border-subtle);
  }

  .brand-logo {
    color: var(--accent);
    flex-shrink: 0;
  }

  .brand-name {
    font-size: var(--text-xl);
    font-weight: 700;
    background: linear-gradient(135deg, #818cf8, #a78bfa, #c084fc);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    letter-spacing: -0.3px;
    line-height: 1.2;
  }

  .brand-tagline {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    letter-spacing: 0.3px;
  }

  .sidebar-section {
    padding: var(--space-3) var(--space-4);
  }

  .sidebar-section-title {
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.8px;
    margin-bottom: var(--space-2);
  }

  .sidebar-action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-3);
    border-radius: var(--radius-lg);
    font-family: var(--font-sans);
    font-size: var(--text-md);
    font-weight: 600;
    cursor: pointer;
    border: none;
    transition: all var(--transition-base);
  }

  .sidebar-action-btn.primary {
    background: linear-gradient(135deg, var(--accent), #a78bfa);
    color: white;
    box-shadow: 0 4px 16px rgba(99, 102, 241, 0.3);
  }

  .sidebar-action-btn.primary:hover {
    transform: translateY(-1px);
    box-shadow: 0 6px 24px rgba(99, 102, 241, 0.4);
  }

  .sidebar-action-btn.primary:active {
    transform: translateY(0);
  }

  /* Guide Steps */
  .guide-step {
    display: flex;
    flex-direction: column;
    width: 100%;
    padding: var(--space-2) var(--space-2);
    border-radius: var(--radius-md);
    cursor: pointer;
    background: none;
    border: none;
    text-align: left;
    font-family: var(--font-sans);
    transition: background var(--transition-fast);
  }

  .guide-step:hover {
    background: var(--surface-card-hover);
  }

  .guide-step-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
  }

  .guide-step-icon {
    font-size: var(--text-md);
    flex-shrink: 0;
  }

  .guide-step-num {
    font-size: var(--text-xs);
    font-weight: 700;
    color: var(--accent);
    font-family: var(--font-mono);
  }

  .guide-step-title {
    flex: 1;
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  .guide-chevron {
    color: var(--text-tertiary);
    transition: transform var(--transition-fast);
    flex-shrink: 0;
  }

  .guide-step.expanded .guide-chevron {
    transform: rotate(180deg);
  }

  .guide-step-desc {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    line-height: 1.5;
    padding: var(--space-2) 0 0 calc(var(--text-md) + var(--space-2) + var(--space-2));
    animation: slideDown 0.2s ease;
  }

  /* Shortcut grid */
  .shortcut-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-1);
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    padding: 2px 0;
  }

  .shortcut-row kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    height: 18px;
    padding: 0 4px;
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    border-radius: 3px;
  }

  .sidebar-footer {
    margin-top: auto;
    padding: var(--space-2) var(--space-4);
    border-top: 1px solid var(--border-subtle);
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .footer-icon-btn {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px;
    border-radius: var(--radius-sm);
    transition: all 0.15s;
  }

  .footer-icon-btn:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.05);
  }

  .sidebar-footer .version-tag {
    margin-left: auto;
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    font-family: var(--font-mono);
    opacity: 0.5;
  }

  /* ===== MAIN CONTENT ===== */
  .home-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    z-index: 1;
    padding: var(--space-8) var(--space-8) var(--space-16);
  }

  .home-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: var(--space-6);
    animation: slideUp 0.4s ease forwards;
  }

  .welcome-title {
    font-size: var(--text-2xl);
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.5px;
  }

  .welcome-subtitle {
    font-size: var(--text-sm);
    color: var(--text-tertiary);
    margin-top: var(--space-1);
  }

  .header-search {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: var(--space-2) var(--space-3);
    min-width: 200px;
    transition: border-color var(--transition-fast);
  }

  .header-search:focus-within {
    border-color: var(--accent);
  }

  .header-search svg {
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .search-input {
    background: none;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    width: 100%;
  }

  .search-input::placeholder {
    color: var(--text-tertiary);
  }

  /* Stats Bar */
  .stats-bar {
    display: flex;
    gap: var(--space-3);
    margin-bottom: var(--space-6);
    animation: slideUp 0.4s ease 0.1s both;
  }

  .stat-chip {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
    background: var(--surface-glass);
    backdrop-filter: blur(12px);
    border: 1px solid var(--surface-glass-border);
    border-radius: var(--radius-full);
    font-size: var(--text-sm);
  }

  .stat-chip svg {
    color: var(--accent);
    opacity: 0.7;
  }

  .stat-value {
    font-weight: 700;
    color: var(--text-primary);
    font-family: var(--font-mono);
  }

  .stat-label {
    color: var(--text-tertiary);
  }

  /* Content */
  .home-content {
    flex: 1;
  }

  .content-section-title {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 1px;
    margin-bottom: var(--space-4);
    animation: slideUp 0.4s ease 0.15s both;
  }

  /* Projects Grid */
  .projects-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: var(--space-4);
  }

  .project-card {
    display: flex;
    flex-direction: column;
    cursor: pointer;
    transition: all 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
    text-align: left;
    font-family: var(--font-sans);
    width: 100%;
    animation: scaleIn 0.35s ease both;
  }

  .project-card:hover {
    transform: translateY(-4px);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255, 255, 255, 0.08) inset;
    border-color: var(--border-default);
  }

  .project-cover {
    position: relative;
    height: 100px;
    overflow: hidden;
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
    background: var(--bg-tertiary);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .project-thumb-img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.5s cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  .project-card:hover .project-thumb-img {
    transform: scale(1.05);
  }

  .cover-gradient {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      135deg,
      rgba(99, 102, 241, 0.12) 0%,
      rgba(168, 85, 247, 0.08) 50%,
      rgba(236, 72, 153, 0.06) 100%
    );
  }

  .cover-icon {
    position: relative;
    z-index: 1;
  }

  .project-item-count {
    position: absolute;
    bottom: var(--space-2);
    right: var(--space-2);
    display: flex;
    align-items: baseline;
    gap: 3px;
    padding: 2px var(--space-2);
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(8px);
    border-radius: var(--radius-full);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 700;
    color: white;
    z-index: 1;
  }

  .count-label {
    font-weight: 400;
    opacity: 0.6;
    font-size: 9px;
  }

  .project-body {
    padding: var(--space-3) var(--space-4) var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .project-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .project-name {
    font-size: var(--text-md);
    font-weight: 600;
    color: var(--text-primary);
    flex: 1;
    min-width: 0;
  }

  .project-actions {
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity var(--transition-fast);
  }

  .project-card:hover .project-actions {
    opacity: 1;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    background: none;
    border-radius: var(--radius-sm);
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all var(--transition-fast);
    padding: 0;
  }

  .action-btn:hover {
    background: var(--surface-card-hover);
    color: var(--text-primary);
  }

  .action-btn.danger:hover {
    color: var(--color-buang);
  }

  .project-path {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    font-family: var(--font-mono);
  }

  .project-meta {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-1);
  }

  .meta-time {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    opacity: 0.7;
  }

  /* Empty search */
  .empty-search {
    text-align: center;
    padding: var(--space-12);
    color: var(--text-tertiary);
    font-size: var(--text-sm);
  }

  /* ===== Onboarding ===== */
  .onboarding-card {
    max-width: 600px;
    margin: var(--space-8) auto 0;
    padding: var(--space-10);
    text-align: center;
    animation: scaleIn 0.5s ease;
  }

  .onboarding-icon {
    color: var(--accent);
    opacity: 0.6;
    margin-bottom: var(--space-4);
  }

  .onboarding-title {
    font-size: var(--text-xl);
    font-weight: 700;
    color: var(--text-primary);
    margin-bottom: var(--space-2);
  }

  .onboarding-desc {
    color: var(--text-tertiary);
    font-size: var(--text-sm);
    line-height: 1.6;
    margin-bottom: var(--space-6);
    max-width: 400px;
    margin-left: auto;
    margin-right: auto;
  }

  .onboarding-cta {
    margin-bottom: var(--space-8);
    padding: var(--space-3) var(--space-8);
    font-size: var(--text-md);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-glow);
    transition: all var(--transition-base);
  }

  .onboarding-cta:hover {
    box-shadow: 0 0 30px rgba(99, 102, 241, 0.35);
    transform: translateY(-1px);
  }

  .onboarding-features {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-3);
    text-align: left;
    padding-top: var(--space-4);
    border-top: 1px solid var(--border-subtle);
  }

  .feature-item {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    line-height: 1.4;
  }

  .feature-icon {
    font-size: var(--text-md);
    flex-shrink: 0;
  }
</style>
