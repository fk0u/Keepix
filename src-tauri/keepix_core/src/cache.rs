// ============================================================================
// Keepix — Image Cache (LRU, Thread-Safe)
// High-performance in-memory cache for image bytes, inspired by Adobe Bridge
// ============================================================================

use base64::Engine as _;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Default cache capacity (number of images kept in memory)
const DEFAULT_CAPACITY: usize = 300;

/// Thread-safe wrapper around the LRU cache
pub struct CacheState {
    pub cache: Mutex<LruCache<String, CachedImage>>,
}

/// A cached image entry containing the pre-encoded base64 data URI string
#[derive(Clone)]
pub struct CachedImage {
    pub data_uri: Arc<String>,
}

impl CacheState {
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(DEFAULT_CAPACITY).unwrap());
        Self {
            cache: Mutex::new(LruCache::new(cap)),
        }
    }

    /// Resize the cache dynamically at runtime
    pub fn resize(&self, capacity: usize) {
        if let Ok(mut lru) = self.cache.lock() {
            if let Some(cap) = NonZeroUsize::new(capacity) {
                lru.resize(cap);
            }
        }
    }
}

/// Detect MIME type from file extension
fn detect_mime(path: &str) -> &'static str {
    let lower = path.to_lowercase();
    if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".bmp") {
        "image/bmp"
    } else if lower.ends_with(".tiff") || lower.ends_with(".tif") {
        "image/tiff"
    } else if lower.ends_with(".avif") {
        "image/avif"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "application/octet-stream"
    }
}

/// Read an image file and return a base64-encoded data URI.
/// Uses the LRU cache to avoid redundant disk I/O and base64 encoding overhead.
pub fn read_image_as_data_uri(cache: &CacheState, file_path: &str) -> Result<String, String> {
    let normalized = file_path.replace('\\', "/");

    // Check cache first
    {
        let mut lru = cache
            .cache
            .lock()
            .map_err(|e| format!("Cache lock error: {}", e))?;

        if let Some(entry) = lru.get(&normalized) {
            return Ok((*entry.data_uri).clone());
        }
    }

    // Cache miss — read from disk
    let path = Path::new(&normalized);
    let resolved_path = if path.exists() {
        path
    } else {
        let orig_path = Path::new(file_path);
        if !orig_path.exists() {
            return Err(format!("File not found: {}", file_path));
        }
        orig_path
    };

    let data = std::fs::read(resolved_path)
        .map_err(|e| format!("Failed to read {}: {}", file_path, e))?;
    let mime = detect_mime(file_path).to_string();
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    let uri = format!("data:{};base64,{}", mime, b64);
    let cached_uri = Arc::new(uri);

    // Insert into cache
    {
        let mut lru = cache
            .cache
            .lock()
            .map_err(|e| format!("Cache lock error: {}", e))?;
        lru.put(
            normalized,
            CachedImage {
                data_uri: Arc::clone(&cached_uri),
            },
        );
    }

    Ok((*cached_uri).clone())
}

/// Preload multiple images into cache in parallel using rayon.
/// Performs file reading and base64 encoding on rayon threadpool to avoid blocking main thread.
pub fn preload_batch(cache: &CacheState, paths: &[String]) -> usize {
    use rayon::prelude::*;

    let results: Vec<(String, Option<CachedImage>)> = paths
        .par_iter()
        .map(|p| {
            let normalized = p.replace('\\', "/");
            let path = Path::new(p);
            if !path.exists() {
                return (normalized, None);
            }
            match std::fs::read(path) {
                Ok(data) => {
                    let mime = detect_mime(p).to_string();
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
                    let uri = format!("data:{};base64,{}", mime, b64);
                    (
                        normalized,
                        Some(CachedImage {
                            data_uri: Arc::new(uri),
                        }),
                    )
                }
                Err(_) => (normalized, None),
            }
        })
        .collect();

    let mut count = 0;
    if let Ok(mut lru) = cache.cache.lock() {
        for (key, value) in results {
            if let Some(img) = value {
                if !lru.contains(&key) {
                    lru.put(key, img);
                    count += 1;
                }
            }
        }
    }

    count
}

/// Clear the entire image cache
pub fn clear(cache: &CacheState) {
    if let Ok(mut lru) = cache.cache.lock() {
        lru.clear();
    }
}
