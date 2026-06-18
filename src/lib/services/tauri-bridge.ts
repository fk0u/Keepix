// ============================================================================
// Keepix — Tauri IPC Bridge
// Type-safe wrappers around Tauri invoke() calls
// ============================================================================

import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  Project,
  MediaItem,
  Category,
  ExifData,
  ScanProgress,
  ScanResult,
  CategoryStats,
  PaginatedMedia,
  ExifFilters,
  ExportProgress,
} from '$lib/types';

// ============================================================================
// Project commands
// ============================================================================

/** List all projects */
export async function getProjects(): Promise<Project[]> {
  return invoke<Project[]>('get_projects');
}

/** Create a new project */
export async function createProject(name: string, rootPath: string): Promise<Project> {
  return invoke<Project>('create_project', { name, rootPath });
}

/** Get a project by ID */
export async function getProject(projectId: string): Promise<Project> {
  return invoke<Project>('get_project', { projectId });
}

/** Delete a project */
export async function deleteProject(projectId: string): Promise<void> {
  return invoke<void>('delete_project', { projectId });
}

// ============================================================================
// Scan commands
// ============================================================================

/** Scan a folder for media files and import them */
export async function scanFolder(projectId: string, folderPath: string): Promise<ScanResult> {
  return invoke<ScanResult>('scan_folder', { projectId, folderPath });
}

/** Listen for scan progress events */
export async function onScanProgress(
  callback: (progress: ScanProgress) => void
): Promise<UnlistenFn> {
  return listen<ScanProgress>('scan-progress', (event) => {
    callback(event.payload);
  });
}

// ============================================================================
// Media commands
// ============================================================================

/** Get paginated media items with filters */
export async function getMediaItems(
  projectId: string,
  page: number = 0,
  pageSize: number = 200,
  categoryId: number | null = null,
  starRating: number | null = null,
  uncategorizedOnly: boolean = false,
  cameraModel: string | null = null,
  lensModel: string | null = null,
  colorLabel: string | null = null,
  sortBy: string = 'name',
  sortOrder: string = 'asc',
): Promise<PaginatedMedia> {
  return invoke<PaginatedMedia>('get_media_items', {
    projectId,
    page,
    pageSize,
    categoryId,
    starRating,
    uncategorizedOnly,
    cameraModel,
    lensModel,
    colorLabel,
    sortBy,
    sortOrder,
  });
}

/** Get unique EXIF filter options for a project */
export async function getExifFilters(projectId: string): Promise<ExifFilters> {
  return invoke<ExifFilters>('get_exif_filters', { projectId });
}

/** Get a single media item */
export async function getSingleMedia(mediaId: string): Promise<MediaItem> {
  return invoke<MediaItem>('get_single_media', { mediaId });
}

/** Set the category for a media item */
export async function setCategory(
  mediaId: string,
  projectId: string,
  categoryId: number | null
): Promise<void> {
  return invoke<void>('set_category', { mediaId, projectId, categoryId });
}

/** Set star rating for a media item */
export async function setStarRating(mediaId: string, rating: number): Promise<void> {
  return invoke<void>('set_star_rating', { mediaId, rating });
}

/** Set color label for a media item */
export async function setColorLabel(mediaId: string, color: string | null): Promise<void> {
  return invoke<void>('set_color_label', { mediaId, color });
}

/** Get EXIF metadata for a media item */
export async function getMetadata(mediaId: string): Promise<ExifData> {
  return invoke<ExifData>('get_metadata', { mediaId });
}

// ============================================================================
// Undo
// ============================================================================

/** Undo the last action */
export async function undoLastAction(projectId: string): Promise<string | null> {
  return invoke<string | null>('undo_last_action', { projectId });
}

// ============================================================================
// Categories & Stats
// ============================================================================

/** Get all categories */
export async function getCategories(): Promise<Category[]> {
  return invoke<Category[]>('get_categories');
}

/** Get category statistics for a project */
export async function getCategoryStats(projectId: string): Promise<CategoryStats[]> {
  return invoke<CategoryStats[]>('get_category_stats', { projectId });
}

// ============================================================================
// Batch operations
// ============================================================================

/** Batch set category for multiple media items */
export async function batchSetCategory(
  mediaIds: string[],
  projectId: string,
  categoryId: number | null
): Promise<number> {
  return invoke<number>('batch_set_category', { mediaIds, projectId, categoryId });
}

// ============================================================================
// Dialog helpers
// ============================================================================

/** Open a folder picker dialog */
export async function pickFolder(): Promise<string | null> {
  const result = await open({
    directory: true,
    multiple: false,
    title: 'Select a folder to import',
  });
  return result as string | null;
}

// ============================================================================
// Utility
// ============================================================================

/** Convert a local file path to an asset URL for display in the webview */
export function toAssetUrl(filePath: string): string {
  if (!filePath) return '';
  // Normalize Windows backslashes to forward slashes for URL compatibility on Windows
  const normalizedPath = filePath.replace(/\\/g, '/');
  return convertFileSrc(normalizedPath);
}

// ============================================================================
// Export commands
// ============================================================================

/** Trigger async export of culled items */
export async function exportMediaItems(
  projectId: string,
  targetDir: string,
  categories: number[],
  exportUncategorized: boolean,
  actionType: 'copy' | 'move' | 'list',
  conflictBehavior: 'overwrite' | 'skip' | 'rename'
): Promise<number> {
  return invoke<number>('export_media_items', {
    projectId,
    targetDir,
    categories,
    exportUncategorized,
    actionType,
    conflictBehavior,
  });
}

/** Listen for export progress events */
export async function onExportProgress(
  callback: (progress: ExportProgress) => void
): Promise<UnlistenFn> {
  return listen<ExportProgress>('export-progress', (event) => {
    callback(event.payload);
  });
}

// ============================================================================
// Image Cache — high-performance image loading
// ============================================================================

// Re-export the image cache service as the primary way to load images
export { getImageDataUri, prefetchImages, clearImageCache } from '$lib/services/image-cache';

