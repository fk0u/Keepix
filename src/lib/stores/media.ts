// ============================================================================
// Keepix — Media State Store (Svelte 5 Runes)
// ============================================================================

import { writable, derived, get } from 'svelte/store';
import type { MediaItem, ViewMode, SortBy, SortOrder, Category, CategoryStats, ExifFilters, DiagnosticsMode, BurstGroup } from '$lib/types';
import * as bridge from '$lib/services/tauri-bridge';
import { clearImageCache } from '$lib/services/image-cache';
// ---- Core state ----
export const mediaItems = writable<MediaItem[]>([]);
export const totalItems = writable<number>(0);
export const currentPage = writable<number>(0);
export const pageSize = writable<number>(500);
export const selectedIndex = writable<number>(0);
export const selectedIds = writable<Set<string>>(new Set());
export const viewMode = writable<ViewMode>('grid');
export const isLoading = writable<boolean>(false);

// ---- Filters ----
export const categoryFilter = writable<number | null>(null);
export const starFilter = writable<number | null>(null);
export const uncategorizedOnly = writable<boolean>(false);
export const cameraModelFilter = writable<string | null>(null);
export const lensModelFilter = writable<string | null>(null);
export const colorLabelFilter = writable<string | null>(null);

// ---- Diagnostics, Histogram, Auto-Advance, and Burst Grouping ----
export const diagnosticsMode = writable<DiagnosticsMode>('none');
export const showHistogram = writable<boolean>(false);
export const autoAdvance = writable<boolean>(true);
export const groupBursts = writable<boolean>(false);

// ---- Sort ----
export const sortBy = writable<SortBy>('name');
export const sortOrder = writable<SortOrder>('asc');

// ---- Compare & Zoom ----
export const compareMode = writable<'single' | '2-up' | '4-up'>('single');
export const syncZoom = writable<boolean>(true);

// ---- Categories ----
export const categories = writable<Category[]>([]);
export const categoryStats = writable<CategoryStats[]>([]);
export const exifFilters = writable<ExifFilters>({ camera_models: [], lens_models: [], color_labels: [] });

// ---- Derived ----
export const currentItem = derived(
  [mediaItems, selectedIndex],
  ([$items, $idx]) => $items[$idx] ?? null
);

export const hasItems = derived(mediaItems, ($items) => $items.length > 0);

export const selectedCount = derived(selectedIds, ($ids) => $ids.size);

export const displayItems = derived(
  [mediaItems, groupBursts],
  ([$items, $group]) => {
    if (!$group) return $items;
    
    // Group consecutive items by date_taken difference <= 3 seconds
    const groups: (MediaItem | BurstGroup)[] = [];
    let currentGroup: BurstGroup | null = null;
    
    for (let i = 0; i < $items.length; i++) {
      const item = $items[i];
      if (item.file_type !== 'photo' || !item.date_taken) {
        if (currentGroup) {
          groups.push(currentGroup);
          currentGroup = null;
        }
        groups.push(item);
        continue;
      }
      
      // EXIF date format is YYYY:MM:DD HH:MM:SS, let's normalize it to YYYY-MM-DD HH:MM:SS
      const normalizedDate = item.date_taken.replace(/:/g, (match, offset) => {
        // Only replace the first two colons which are part of YYYY:MM:DD
        return offset < 10 ? '-' : match;
      });
      const itemTime = Date.parse(normalizedDate);
      if (isNaN(itemTime)) {
        if (currentGroup) {
          groups.push(currentGroup);
          currentGroup = null;
        }
        groups.push(item);
        continue;
      }
      
      if (currentGroup) {
        const leadNormalizedDate = currentGroup.leadItem.date_taken!.replace(/:/g, (match, offset) => {
          return offset < 10 ? '-' : match;
        });
        const leadTime = Date.parse(leadNormalizedDate);
        const diffSeconds = Math.abs(itemTime - leadTime) / 1000;
        
        if (diffSeconds <= 3) {
          currentGroup.items.push(item);
        } else {
          groups.push(currentGroup);
          currentGroup = {
            id: `burst-${item.id}`,
            leadItem: item,
            items: [item]
          };
        }
      } else {
        currentGroup = {
          id: `burst-${item.id}`,
          leadItem: item,
          items: [item]
        };
      }
    }
    
    if (currentGroup) {
      groups.push(currentGroup);
    }
    
    return groups;
  }
);

// ============================================================================
// Actions
// ============================================================================

/** Load media items for a project */
export async function loadMediaItems(projectId: string) {
  isLoading.set(true);
  try {
    const page = get(currentPage);
    const size = get(pageSize);
    const catFilter = get(categoryFilter);
    const starF = get(starFilter);
    const uncatOnly = get(uncategorizedOnly);
    const cameraM = get(cameraModelFilter);
    const lensM = get(lensModelFilter);
    const colorL = get(colorLabelFilter);
    const sortField = get(sortBy);
    const sortDir = get(sortOrder);

    const result = await bridge.getMediaItems(
      projectId, page, size, catFilter, starF, uncatOnly,
      cameraM, lensM, colorL, sortField, sortDir
    );
    
    mediaItems.set(result.items);
    totalItems.set(result.total);
  } catch (err) {
    console.error('Failed to load media items:', err);
  } finally {
    isLoading.set(false);
  }
}

/** Load unique EXIF filters for a project */
export async function loadExifFilters(projectId: string) {
  try {
    const filters = await bridge.getExifFilters(projectId);
    exifFilters.set(filters);
  } catch (err) {
    console.error('Failed to load EXIF filters:', err);
  }
}

/** Load categories */
export async function loadCategories() {
  try {
    const cats = await bridge.getCategories();
    categories.set(cats);
  } catch (err) {
    console.error('Failed to load categories:', err);
  }
}

/** Load category statistics */
export async function loadCategoryStats(projectId: string) {
  try {
    const stats = await bridge.getCategoryStats(projectId);
    categoryStats.set(stats);
  } catch (err) {
    console.error('Failed to load stats:', err);
  }
}

/** Set category for the current item */
export async function setCategoryForItem(
  mediaId: string,
  projectId: string,
  categoryId: number | null
) {
  try {
    await bridge.setCategory(mediaId, projectId, categoryId);

    // Update local state immediately
    mediaItems.update(items =>
      items.map(item =>
        item.id === mediaId ? { ...item, category_id: categoryId } : item
      )
    );

    // Refresh stats
    await loadCategoryStats(projectId);
  } catch (err) {
    console.error('Failed to set category:', err);
  }
}

/** Set star rating for an item */
export async function setStarRatingForItem(mediaId: string, rating: number) {
  try {
    await bridge.setStarRating(mediaId, rating);

    mediaItems.update(items =>
      items.map(item =>
        item.id === mediaId ? { ...item, star_rating: rating } : item
      )
    );
  } catch (err) {
    console.error('Failed to set rating:', err);
  }
}

/** Set color label for an item */
export async function setColorLabelForItem(mediaId: string, color: string | null) {
  try {
    await bridge.setColorLabel(mediaId, color);

    mediaItems.update(items =>
      items.map(item =>
        item.id === mediaId ? { ...item, color_label: color } : item
      )
    );
  } catch (err) {
    console.error('Failed to set color label:', err);
  }
}

/** Navigate to next item */
export function navigateNext() {
  const items = get(mediaItems);
  selectedIndex.update(idx => Math.min(idx + 1, items.length - 1));
}

/** Navigate to previous item */
export function navigatePrev() {
  selectedIndex.update(idx => Math.max(idx - 1, 0));
}

/** Navigate to specific index */
export function navigateTo(index: number) {
  const items = get(mediaItems);
  if (index >= 0 && index < items.length) {
    selectedIndex.set(index);
  }
}

/** Toggle view mode */
export function toggleViewMode() {
  viewMode.update(mode => mode === 'grid' ? 'preview' : 'grid');
}

/** Set a category filter */
export function setFilterCategory(catId: number | null) {
  categoryFilter.set(catId);
  uncategorizedOnly.set(false);
  currentPage.set(0);
  selectedIndex.set(0);
}

/** Show only uncategorized items */
export function setFilterUncategorized() {
  categoryFilter.set(null);
  uncategorizedOnly.set(true);
  currentPage.set(0);
  selectedIndex.set(0);
}

/** Clear all filters */
export function clearFilters() {
  categoryFilter.set(null);
  starFilter.set(null);
  uncategorizedOnly.set(false);
  cameraModelFilter.set(null);
  lensModelFilter.set(null);
  colorLabelFilter.set(null);
  currentPage.set(0);
  selectedIndex.set(0);
}

/** Reset store state */
export function resetMediaStore() {
  mediaItems.set([]);
  totalItems.set(0);
  currentPage.set(0);
  selectedIndex.set(0);
  selectedIds.set(new Set());
  categoryFilter.set(null);
  starFilter.set(null);
  uncategorizedOnly.set(false);
  cameraModelFilter.set(null);
  lensModelFilter.set(null);
  colorLabelFilter.set(null);
  diagnosticsMode.set('none');
  showHistogram.set(false);
  autoAdvance.set(true);
  groupBursts.set(false);
  exifFilters.set({ camera_models: [], lens_models: [], color_labels: [] });
  compareMode.set('single');
  categoryStats.set([]);
  // Clear the image cache when switching projects
  clearImageCache();
}
