// ============================================================================
// Keepix — Media State Store (Svelte 5 Runes)
// ============================================================================

import { writable, derived, get } from 'svelte/store';
import type { MediaItem, ViewMode, SortBy, SortOrder, Category, CategoryStats } from '$lib/types';
import * as bridge from '$lib/services/tauri-bridge';

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

// ---- Sort ----
export const sortBy = writable<SortBy>('name');
export const sortOrder = writable<SortOrder>('asc');

// ---- Categories ----
export const categories = writable<Category[]>([]);
export const categoryStats = writable<CategoryStats[]>([]);

// ---- Derived ----
export const currentItem = derived(
  [mediaItems, selectedIndex],
  ([$items, $idx]) => $items[$idx] ?? null
);

export const hasItems = derived(mediaItems, ($items) => $items.length > 0);

export const selectedCount = derived(selectedIds, ($ids) => $ids.size);

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

    const result = await bridge.getMediaItems(
      projectId, page, size, catFilter, starF, uncatOnly
    );
    
    mediaItems.set(result.items);
    totalItems.set(result.total);
  } catch (err) {
    console.error('Failed to load media items:', err);
  } finally {
    isLoading.set(false);
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
  categoryStats.set([]);
}
