// ============================================================================
// Keepix — Image Cache (LRU, Thread-Safe)
// High-performance in-memory cache for image bytes, inspired by Adobe Bridge
// ============================================================================

use base64::Engine as _;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Mutex;

/// Default cache capacity (number of images kept in memory)
const DEFAULT_CAPACITY: usize = 300;

/// Thread-safe wrapper around the LRU cache
pub struct CacheState {
    pub cache: Mutex<LruCache<String, CachedImage>>,
}

/// A cached image entry containing the raw bytes and MIME type
#[derive(Clone)]
pub struct CachedImage {
    pub data: Vec<u8>,
    pub mime: String,
}

impl CacheState {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(DEFAULT_CAPACITY).unwrap(),
            )),
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
/// Uses the LRU cache to avoid redundant disk I/O.
pub fn read_image_as_data_uri(cache: &CacheState, file_path: &str) -> Result<String, String> {
    let normalized = file_path.replace('\\', "/");

    // Check cache first
    {
        let mut lru = cache
            .cache
            .lock()
            .map_err(|e| format!("Cache lock error: {}", e))?;

        if let Some(entry) = lru.get(&normalized) {
            let b64 = base64::engine::general_purpose::STANDARD.encode(&entry.data);
            return Ok(format!("data:{};base64,{}", entry.mime, b64));
        }
    }

    // Cache miss — read from disk
    let path = Path::new(&normalized);
    if !path.exists() {
        // Also try the original path (with backslashes on Windows)
        let orig_path = Path::new(file_path);
        if !orig_path.exists() {
            return Err(format!("File not found: {}", file_path));
        }
        let data = std::fs::read(orig_path)
            .map_err(|e| format!("Failed to read {}: {}", file_path, e))?;
        let mime = detect_mime(file_path).to_string();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
        let uri = format!("data:{};base64,{}", mime, b64);

        // Insert into cache
        {
            let mut lru = cache
                .cache
                .lock()
                .map_err(|e| format!("Cache lock error: {}", e))?;
            lru.put(
                normalized,
                CachedImage {
                    data,
                    mime,
                },
            );
        }

        return Ok(uri);
    }

    let data =
        std::fs::read(&normalized).map_err(|e| format!("Failed to read {}: {}", normalized, e))?;
    let mime = detect_mime(&normalized).to_string();
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    let uri = format!("data:{};base64,{}", mime, b64);

    // Insert into cache
    {
        let mut lru = cache
            .cache
            .lock()
            .map_err(|e| format!("Cache lock error: {}", e))?;
        lru.put(
            normalized,
            CachedImage {
                data,
                mime,
            },
        );
    }

    Ok(uri)
}

/// Preload multiple images into cache in parallel using rayon.
/// Returns the number of images successfully cached.
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
                    (normalized, Some(CachedImage { data, mime }))
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
