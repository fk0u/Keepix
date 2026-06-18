use serde::Serialize;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Supported photo extensions
const PHOTO_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "tiff", "tif", "bmp", "heic", "heif",
    "cr2", "cr3", "nef", "arw", "orf", "rw2", "dng", "raf", "pef",
];

/// Supported video extensions
const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "avi", "mkv", "webm", "m4v", "wmv", "flv", "3gp",
];

/// A discovered file from scanning
#[derive(Debug, Clone, Serialize)]
pub struct ScannedFile {
    pub file_path: String,
    pub file_name: String,
    pub file_type: String, // "photo" or "video"
    pub file_size: i64,
    pub modified_at: Option<String>,
}

/// Progress information during scanning
#[derive(Debug, Clone, Serialize)]
pub struct ScanProgress {
    pub total_found: usize,
    pub current_file: String,
    pub phase: String, // "scanning" | "processing" | "complete"
}

/// Scan a directory recursively for photo and video files
pub fn scan_directory(root_path: &Path) -> Vec<ScannedFile> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip hidden files and directories
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }

        if !path.is_file() {
            continue;
        }

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            let file_type = if PHOTO_EXTENSIONS.contains(&ext_lower.as_str()) {
                "photo"
            } else if VIDEO_EXTENSIONS.contains(&ext_lower.as_str()) {
                "video"
            } else {
                continue;
            };

            let file_size = entry.metadata().map(|m| m.len() as i64).unwrap_or(0);

            let modified_at = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Local> = t.into();
                    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                });

            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            files.push(ScannedFile {
                file_path: path.to_string_lossy().to_string(),
                file_name,
                file_type: file_type.to_string(),
                file_size,
                modified_at,
            });
        }
    }

    files
}

/// Get the thumbnail cache directory for a project (stored in workspace/.keepix/previews)
pub fn get_thumbnail_dir(project_root: &Path) -> PathBuf {
    let dir = project_root.join(".keepix").join("previews");
    std::fs::create_dir_all(&dir).ok();
    dir
}

/// Determine file type from extension
pub fn get_file_type(path: &Path) -> Option<&'static str> {
    path.extension()
        .and_then(|e| e.to_str())
        .and_then(|ext| {
            let ext_lower = ext.to_lowercase();
            if PHOTO_EXTENSIONS.contains(&ext_lower.as_str()) {
                Some("photo")
            } else if VIDEO_EXTENSIONS.contains(&ext_lower.as_str()) {
                Some("video")
            } else {
                None
            }
        })
}
