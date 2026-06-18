// ============================================================================
// Keepix — Frontend Image Cache Service
// LRU-based cache that serves images as data URIs via Rust backend
// Inspired by Adobe Bridge's aggressive caching strategy
// ============================================================================

import { invoke } from '@tauri-apps/api/core';

/** LRU Cache capacity for frontend-side data URIs */
const FRONTEND_CACHE_CAPACITY = 500;

/** In-flight request deduplication map */
const inflight = new Map<string, Promise<string>>();

/**
 * Simple LRU Cache using a Map (Map preserves insertion order).
 * When capacity is exceeded, the least-recently-used entry is evicted.
 */
class LRUCache<K, V> {
  private cache = new Map<K, V>();
  private readonly capacity: number;

  constructor(capacity: number) {
    this.capacity = capacity;
  }

  get(key: K): V | undefined {
    if (!this.cache.has(key)) return undefined;
    // Move to end (most recently used)
    const value = this.cache.get(key)!;
    this.cache.delete(key);
    this.cache.set(key, value);
    return value;
  }

  set(key: K, value: V): void {
    if (this.cache.has(key)) {
      this.cache.delete(key);
    } else if (this.cache.size >= this.capacity) {
      // Evict the least-recently-used (first entry)
      const firstKey = this.cache.keys().next().value;
      if (firstKey !== undefined) {
        this.cache.delete(firstKey);
      }
    }
    this.cache.set(key, value);
  }

  has(key: K): boolean {
    return this.cache.has(key);
  }

  delete(key: K): void {
    this.cache.delete(key);
  }

  clear(): void {
    this.cache.clear();
  }

  get size(): number {
    return this.cache.size;
  }
}

/** The frontend LRU cache storing data URI strings */
const cache = new LRUCache<string, string>(FRONTEND_CACHE_CAPACITY);

/**
 * Get an image as a data URI. Uses multi-tier caching:
 * 1. Check frontend JS LRU cache
 * 2. If miss, call Rust backend (which has its own LRU cache + disk read)
 * 3. Cache the result in frontend cache
 * 
 * Deduplicates concurrent requests for the same path.
 */
export async function getImageDataUri(path: string | null): Promise<string> {
  if (!path) return '';
  
  // Normalize path
  const normalizedPath = path.replace(/\\/g, '/');

  // Check frontend cache
  const cached = cache.get(normalizedPath);
  if (cached) return cached;

  // Check if there's already an in-flight request for this path
  const existing = inflight.get(normalizedPath);
  if (existing) return existing;

  // Create the request and store it for deduplication
  const request = (async () => {
    try {
      const dataUri = await invoke<string>('read_image_base64', { path: normalizedPath });
      cache.set(normalizedPath, dataUri);
      return dataUri;
    } catch (err) {
      console.warn(`Failed to load image: ${normalizedPath}`, err);
      return '';
    } finally {
      inflight.delete(normalizedPath);
    }
  })();

  inflight.set(normalizedPath, request);
  return request;
}

/**
 * Prefetch multiple images into both backend and frontend caches.
 * Useful for preloading adjacent thumbnails during grid scrolling.
 */
export async function prefetchImages(paths: (string | null)[]): Promise<void> {
  const validPaths = paths.filter((p): p is string => !!p);
  if (validPaths.length === 0) return;

  // Filter out already-cached paths
  const uncached = validPaths.filter(p => !cache.has(p.replace(/\\/g, '/')));
  if (uncached.length === 0) return;

  try {
    // Tell Rust backend to preload into its LRU cache
    await invoke('preload_images', { paths: uncached });

    // Then fetch them into frontend cache (batch, non-blocking)
    const promises = uncached.map(p => getImageDataUri(p));
    await Promise.allSettled(promises);
  } catch (err) {
    console.warn('Prefetch failed:', err);
  }
}

/**
 * Clear the entire image cache (both frontend and backend).
 * Call when switching projects.
 */
export async function clearImageCache(): Promise<void> {
  cache.clear();
  inflight.clear();
  try {
    await invoke('clear_image_cache');
  } catch (err) {
    console.warn('Failed to clear backend cache:', err);
  }
}

/**
 * Get the current frontend cache size (for debugging/stats).
 */
export function getCacheSize(): number {
  return cache.size;
}
