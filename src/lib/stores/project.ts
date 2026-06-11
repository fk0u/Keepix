// ============================================================================
// Keepix — Project State Store
// ============================================================================

import { writable, get } from 'svelte/store';
import type { Project, ScanProgress } from '$lib/types';
import * as bridge from '$lib/services/tauri-bridge';

// ---- State ----
export const currentProject = writable<Project | null>(null);
export const projects = writable<Project[]>([]);
export const scanProgress = writable<ScanProgress | null>(null);
export const isScanning = writable<boolean>(false);

// ============================================================================
// Actions
// ============================================================================

/** Load all projects */
export async function loadProjects() {
  try {
    const list = await bridge.getProjects();
    projects.set(list);
  } catch (err) {
    console.error('Failed to load projects:', err);
  }
}

/** Create a new project from a folder */
export async function createProjectFromFolder(folderPath: string): Promise<Project | null> {
  try {
    // Extract folder name for project name
    const parts = folderPath.replace(/\\/g, '/').split('/');
    const name = parts[parts.length - 1] || 'Untitled';

    const project = await bridge.createProject(name, folderPath);
    await loadProjects();
    currentProject.set(project);
    return project;
  } catch (err) {
    console.error('Failed to create project:', err);
    return null;
  }
}

/** Open a folder picker and create a project */
export async function openFolderAndCreateProject(): Promise<Project | null> {
  const folderPath = await bridge.pickFolder();
  if (!folderPath) return null;
  return createProjectFromFolder(folderPath);
}

/** Set current project */
export function setCurrentProject(project: Project) {
  currentProject.set(project);
}

/** Delete a project */
export async function removeProject(projectId: string) {
  try {
    await bridge.deleteProject(projectId);
    await loadProjects();

    const current = get(currentProject);
    if (current?.id === projectId) {
      currentProject.set(null);
    }
  } catch (err) {
    console.error('Failed to delete project:', err);
  }
}

/** Start scanning a folder for the current project */
export async function startScan(projectId: string, folderPath: string) {
  isScanning.set(true);
  scanProgress.set({
    phase: 'scanning',
    current: 0,
    total: 0,
    current_file: 'Starting scan...',
  });

  try {
    // Listen for progress events
    const unlisten = await bridge.onScanProgress((progress) => {
      scanProgress.set(progress);
      if (progress.phase === 'complete') {
        isScanning.set(false);
        unlisten();
      }
    });

    // Start the scan
    await bridge.scanFolder(projectId, folderPath);
  } catch (err) {
    console.error('Scan failed:', err);
    isScanning.set(false);
    scanProgress.set(null);
  }
}
