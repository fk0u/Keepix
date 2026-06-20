use crate::db::{self, DbState, MediaItem};
use crate::metadata;
use crate::scanner;
use crate::thumbnail;
use serde::Serialize;
use std::path::Path;
use tauri::{AppHandle, Emitter, Manager};

// ============================================================================
// Response types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ScanResult {
    pub project_id: String,
    pub total_files: usize,
    pub photos: usize,
    pub videos: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanProgressEvent {
    pub phase: String,
    pub current: usize,
    pub total: usize,
    pub current_file: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportProgressEvent {
    pub phase: String,
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub status: String,
}


#[derive(Debug, Serialize)]
pub struct PaginatedMedia {
    pub items: Vec<MediaItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

// ============================================================================
// Project commands
// ============================================================================

#[tauri::command]
pub fn get_projects(state: tauri::State<'_, DbState>) -> Result<Vec<db::Project>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::list_projects(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_project(
    state: tauri::State<'_, DbState>,
    name: String,
    root_path: String,
) -> Result<db::Project, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    db::create_project(&conn, &id, &name, &root_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_project(state: tauri::State<'_, DbState>, project_id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::delete_project(&conn, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_project(
    state: tauri::State<'_, DbState>,
    project_id: String,
) -> Result<db::Project, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_project(&conn, &project_id).map_err(|e| e.to_string())
}

// ============================================================================
// Scan & import commands
// ============================================================================

#[tauri::command]
pub async fn scan_folder(
    app: AppHandle,
    state: tauri::State<'_, DbState>,
    project_id: String,
    folder_path: String,
) -> Result<ScanResult, String> {
    let path = Path::new(&folder_path);
    if !path.exists() || !path.is_dir() {
        return Err("Invalid folder path".to_string());
    }

    // Phase 1: Scan directory
    app.emit("scan-progress", ScanProgressEvent {
        phase: "scanning".to_string(),
        current: 0,
        total: 0,
        current_file: "Scanning directory...".to_string(),
    })
    .ok();

    let scanned_files = scanner::scan_directory(path);
    let total = scanned_files.len();
    let photos = scanned_files.iter().filter(|f| f.file_type == "photo").count();
    let videos = scanned_files.iter().filter(|f| f.file_type == "video").count();

    // Phase 2: Insert into database
    app.emit("scan-progress", ScanProgressEvent {
        phase: "importing".to_string(),
        current: 0,
        total,
        current_file: "Saving to database...".to_string(),
    })
    .ok();

    let media_items: Vec<MediaItem> = scanned_files
        .iter()
        .map(|f| MediaItem {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: project_id.clone(),
            file_path: f.file_path.clone(),
            file_name: f.file_name.clone(),
            file_type: f.file_type.clone(),
            file_size: f.file_size,
            width: None,
            height: None,
            category_id: None,
            star_rating: 0,
            color_label: None,
            thumbnail_path: None,
            preview_path: None,
            exif_json: None,
            file_hash: None,
            date_taken: f.modified_at.clone(),
            adjustments: None,
            applied_preset: None,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            updated_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        })
        .collect();

    {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        db::insert_media_batch(&conn, &media_items).map_err(|e| e.to_string())?;
        db::update_project_stats(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    // Get cpu_threads preference setting
    let cpu_threads = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        db::get_setting(&conn, "cpu_threads")
            .unwrap_or(None)
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(4)
    };

    // Phase 3: Generate thumbnails in background
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let thumb_dir = scanner::get_thumbnail_dir(Path::new(&folder_path));
    let app_clone = app.clone();

    // Clone state for the background thread — must be done before moving into closure
    let db_state: DbState = {
        DbState {
            conn: std::sync::Mutex::new(
                rusqlite::Connection::open(
                    app_data_dir.join("keepix.db")
                ).map_err(|e| e.to_string())?
            ),
        }
    };
    let items_for_thumbs = media_items.clone();

    std::thread::spawn(move || {
        let total = items_for_thumbs.len();
        
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(cpu_threads)
            .build();

        if let Ok(pool) = pool {
            use std::sync::atomic::{AtomicUsize, Ordering};
            use rayon::prelude::*;
            let progress_counter = AtomicUsize::new(0);

            pool.install(|| {
                items_for_thumbs.par_iter().for_each(|item| {
                    let result = if item.file_type == "photo" {
                        thumbnail::generate_thumbnails(
                            Path::new(&item.file_path),
                            &thumb_dir,
                            &item.id,
                        )
                    } else {
                        thumbnail::generate_video_placeholder(&thumb_dir, &item.id)
                    };

                    if let Ok(thumb_result) = &result {
                        if let Ok(conn) = db_state.conn.lock() {
                            db::update_thumbnail(
                                &conn,
                                &item.id,
                                &thumb_result.thumbnail_path,
                                Some(&thumb_result.preview_path),
                            )
                            .ok();

                            // Also read and store EXIF for photos
                            if item.file_type == "photo" {
                                let exif = metadata::read_exif(Path::new(&item.file_path));
                                if let Ok(exif_json) = serde_json::to_string(&exif) {
                                    db::update_exif(
                                        &conn,
                                        &item.id,
                                        &exif_json,
                                        exif.width.map(|w| w as i32),
                                        exif.height.map(|h| h as i32),
                                    )
                                    .ok();
                                }
                            }
                        }
                    }

                    // Emit progress
                    let current = progress_counter.fetch_add(1, Ordering::SeqCst) + 1;
                    app_clone
                        .emit(
                            "scan-progress",
                            ScanProgressEvent {
                                phase: "thumbnails".to_string(),
                                current,
                                total,
                                current_file: item.file_name.clone(),
                            },
                        )
                        .ok();
                });
            });
        } else {
            // Fallback to sequential
            for (idx, item) in items_for_thumbs.iter().enumerate() {
                let result = if item.file_type == "photo" {
                    thumbnail::generate_thumbnails(
                        Path::new(&item.file_path),
                        &thumb_dir,
                        &item.id,
                    )
                } else {
                    thumbnail::generate_video_placeholder(&thumb_dir, &item.id)
                };

                if let Ok(thumb_result) = &result {
                    if let Ok(conn) = db_state.conn.lock() {
                        db::update_thumbnail(
                            &conn,
                            &item.id,
                            &thumb_result.thumbnail_path,
                            Some(&thumb_result.preview_path),
                        )
                        .ok();

                        if item.file_type == "photo" {
                            let exif = metadata::read_exif(Path::new(&item.file_path));
                            if let Ok(exif_json) = serde_json::to_string(&exif) {
                                db::update_exif(
                                    &conn,
                                    &item.id,
                                    &exif_json,
                                    exif.width.map(|w| w as i32),
                                    exif.height.map(|h| h as i32),
                                )
                                .ok();
                            }
                        }
                    }
                }

                // Emit progress
                app_clone
                    .emit(
                        "scan-progress",
                        ScanProgressEvent {
                            phase: "thumbnails".to_string(),
                            current: idx + 1,
                            total,
                            current_file: item.file_name.clone(),
                        },
                    )
                    .ok();
            }
        }

        // Emit completion
        app_clone
            .emit(
                "scan-progress",
                ScanProgressEvent {
                    phase: "complete".to_string(),
                    current: total,
                    total,
                    current_file: "Done!".to_string(),
                },
            )
            .ok();
    });

    Ok(ScanResult {
        project_id,
        total_files: total,
        photos,
        videos,
    })
}

// ============================================================================
// Media commands
// ============================================================================

#[tauri::command]
pub fn get_media_items(
    state: tauri::State<'_, DbState>,
    project_id: String,
    page: i64,
    page_size: i64,
    category_id: Option<i32>,
    star_rating: Option<i32>,
    uncategorized_only: bool,
    camera_model: Option<String>,
    lens_model: Option<String>,
    color_label: Option<String>,
    sort_by: String,
    sort_order: String,
) -> Result<PaginatedMedia, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let offset = page * page_size;

    let items = db::get_media_items(
        &conn,
        &project_id,
        category_id,
        star_rating,
        uncategorized_only,
        camera_model.as_deref(),
        lens_model.as_deref(),
        color_label.as_deref(),
        &sort_by,
        &sort_order,
        offset,
        page_size,
    )
    .map_err(|e| e.to_string())?;

    let total = db::get_media_count(
        &conn,
        &project_id,
        category_id,
        star_rating,
        uncategorized_only,
        camera_model.as_deref(),
        lens_model.as_deref(),
        color_label.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    Ok(PaginatedMedia {
        items,
        total,
        page,
        page_size,
    })
}

#[tauri::command]
pub fn get_exif_filters(
    state: tauri::State<'_, DbState>,
    project_id: String,
) -> Result<db::ExifFilters, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_unique_exif_metadata(&conn, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_single_media(
    state: tauri::State<'_, DbState>,
    media_id: String,
) -> Result<MediaItem, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_media_item(&conn, &media_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_category(
    state: tauri::State<'_, DbState>,
    media_id: String,
    project_id: String,
    category_id: Option<i32>,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;

    // Get old value for undo
    let old_category: Option<i32> = db::set_media_category(&conn, &media_id, category_id)
        .map_err(|e| e.to_string())?;

    // Push undo entry
    let old_str = old_category.map(|c: i32| c.to_string());
    let new_str = category_id.map(|c| c.to_string());
    db::push_undo(
        &conn,
        &project_id,
        &media_id,
        "set_category",
        old_str.as_deref(),
        new_str.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn set_star_rating(
    state: tauri::State<'_, DbState>,
    media_id: String,
    rating: i32,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::set_star_rating(&conn, &media_id, rating).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_color_label(
    state: tauri::State<'_, DbState>,
    media_id: String,
    color: Option<String>,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::set_color_label(&conn, &media_id, color.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_metadata(
    state: tauri::State<'_, DbState>,
    media_id: String,
) -> Result<metadata::ExifData, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let item = db::get_media_item(&conn, &media_id).map_err(|e| e.to_string())?;

    // If we have cached EXIF, return it
    if let Some(ref exif_json) = item.exif_json {
        if let Ok(exif) = serde_json::from_str::<metadata::ExifData>(exif_json) {
            return Ok(exif);
        }
    }

    // Otherwise, read it fresh
    let mut exif = metadata::read_exif(Path::new(&item.file_path));
    exif.file_size = Some(item.file_size);
    exif.file_format = Path::new(&item.file_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_uppercase());

    // Cache it
    if let Ok(exif_json) = serde_json::to_string(&exif) {
        db::update_exif(
            &conn,
            &media_id,
            &exif_json,
            exif.width.map(|w| w as i32),
            exif.height.map(|h| h as i32),
        )
        .ok();
    }

    Ok(exif)
}

// ============================================================================
// Undo command
// ============================================================================

#[tauri::command]
pub fn undo_last_action(
    state: tauri::State<'_, DbState>,
    project_id: String,
) -> Result<Option<String>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;

    let entry = db::pop_undo(&conn, &project_id).map_err(|e| e.to_string())?;

    if let Some(entry) = entry {
        match entry.action_type.as_str() {
            "set_category" => {
                let old_cat: Option<i32> = entry.old_value.and_then(|v| v.parse().ok());
                db::set_media_category(&conn, &entry.media_id, old_cat)
                    .map_err(|e| e.to_string())?;
                Ok(Some(format!("Undone category change for {}", entry.media_id)))
            }
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

// ============================================================================
// Category & stats commands
// ============================================================================

#[tauri::command]
pub fn get_categories(state: tauri::State<'_, DbState>) -> Result<Vec<db::Category>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_categories(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_category_stats(
    state: tauri::State<'_, DbState>,
    project_id: String,
) -> Result<Vec<db::CategoryStats>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_category_stats(&conn, &project_id).map_err(|e| e.to_string())
}

// ============================================================================
// Batch operations
// ============================================================================

#[tauri::command]
pub fn batch_set_category(
    state: tauri::State<'_, DbState>,
    media_ids: Vec<String>,
    project_id: String,
    category_id: Option<i32>,
) -> Result<usize, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut count = 0;

    for media_id in &media_ids {
        let old_category: Option<i32> = db::set_media_category(&conn, media_id, category_id)
            .map_err(|e| e.to_string())?;

        let old_str = old_category.map(|c: i32| c.to_string());
        let new_str = category_id.map(|c| c.to_string());
        db::push_undo(
            &conn,
            &project_id,
            media_id,
            "set_category",
            old_str.as_deref(),
            new_str.as_deref(),
        )
        .map_err(|e| e.to_string())?;

        count += 1;
    }

    Ok(count)
}

#[tauri::command]
pub async fn export_media_items(
    app: AppHandle,
    state: tauri::State<'_, DbState>,
    project_id: String,
    target_dir: String,
    categories: Vec<i32>,
    export_uncategorized: bool,
    action_type: String, // "copy" | "move" | "list"
    conflict_behavior: String, // "overwrite" | "skip" | "rename"
) -> Result<usize, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    
    let mut sql = String::from("SELECT file_path, file_name FROM media_items WHERE project_id = ?1 AND (");
    let mut conditions = Vec::new();
    
    if !categories.is_empty() {
        let placeholders: Vec<String> = (1..=categories.len()).map(|i| format!("?{}", i + 1)).collect();
        conditions.push(format!("category_id IN ({})", placeholders.join(",")));
    }
    
    if export_uncategorized {
        conditions.push("category_id IS NULL".to_string());
    }
    
    if conditions.is_empty() {
        return Ok(0); // Nothing to export
    }
    
    sql.push_str(&conditions.join(" OR "));
    sql.push_str(")");
    
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();
    params_vec.push(rusqlite::types::Value::Text(project_id.clone()));
    for cat in &categories {
        params_vec.push(rusqlite::types::Value::Integer(*cat as i64));
    }
    
    #[derive(Clone)]
    struct ExportItem {
        file_path: String,
        file_name: String,
    }
    
    let items = stmt
        .query_map(rusqlite::params_from_iter(params_vec), |row| {
            Ok(ExportItem {
                file_path: row.get(0)?,
                file_name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
        
    let total = items.len();
    if total == 0 {
        return Ok(0);
    }

    // Spawn a background thread for asynchronous exporting
    std::thread::spawn(move || {
        let path = std::path::Path::new(&target_dir);
        if !path.exists() {
            std::fs::create_dir_all(path).ok();
        }

        // List mode target file (if action_type is "list")
        let mut list_file_content = String::new();

        for (idx, item) in items.iter().enumerate() {
            let src = std::path::Path::new(&item.file_path);
            if !src.exists() {
                app.emit(
                    "export-progress",
                    ExportProgressEvent {
                        phase: "exporting".to_string(),
                        current: idx + 1,
                        total,
                        current_file: item.file_name.clone(),
                        status: format!("Source file not found: {}", item.file_path),
                    },
                )
                .ok();
                continue;
            }

            if action_type == "list" {
                list_file_content.push_str(&item.file_path);
                list_file_content.push_str("\n");
                
                app.emit(
                    "export-progress",
                    ExportProgressEvent {
                        phase: "exporting".to_string(),
                        current: idx + 1,
                        total,
                        current_file: item.file_name.clone(),
                        status: "Listing...".to_string(),
                    },
                )
                .ok();
                continue;
            }

            // Conflict resolution
            let mut dest_name = item.file_name.clone();
            let mut dest = path.join(&dest_name);

            if dest.exists() {
                match conflict_behavior.as_str() {
                    "skip" => {
                        app.emit(
                            "export-progress",
                            ExportProgressEvent {
                                phase: "exporting".to_string(),
                                current: idx + 1,
                                total,
                                current_file: item.file_name.clone(),
                                status: "Skipped (already exists)".to_string(),
                            },
                        )
                        .ok();
                        continue;
                    }
                    "rename" => {
                        let stem = dest.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                        let ext = dest.extension().and_then(|e| e.to_str()).unwrap_or("").to_string();
                        let mut counter = 1;
                        while dest.exists() {
                            dest_name = if ext.is_empty() {
                                format!("{}_{}", stem, counter)
                            } else {
                                format!("{}_{}.{}", stem, counter, ext)
                            };
                            dest = path.join(&dest_name);
                            counter += 1;
                        }
                    }
                    _ => {} // "overwrite" -> do nothing
                }
            }

            // Perform file operation
            let result = if action_type == "move" {
                match std::fs::rename(src, &dest) {
                    Ok(_) => Ok(()),
                    Err(_) => {
                        match std::fs::copy(src, &dest) {
                            Ok(_) => {
                                std::fs::remove_file(src).ok();
                                Ok(())
                            }
                            Err(e) => Err(e),
                        }
                    }
                }
            } else {
                std::fs::copy(src, &dest).map(|_| ())
            };

            let status_msg = match result {
                Ok(_) => {
                    if action_type == "move" {
                        "Moved".to_string()
                    } else {
                        "Copied".to_string()
                    }
                }
                Err(e) => format!("Failed: {}", e),
            };

            app.emit(
                "export-progress",
                ExportProgressEvent {
                    phase: "exporting".to_string(),
                    current: idx + 1,
                    total,
                    current_file: item.file_name.clone(),
                    status: status_msg,
                },
            )
            .ok();
        }

        // Write file list in list mode
        if action_type == "list" {
            let list_path = path.join("keepix_export_list.txt");
            std::fs::write(&list_path, list_file_content).ok();
            app.emit(
                "export-progress",
                ExportProgressEvent {
                    phase: "exporting".to_string(),
                    current: total,
                    total,
                    current_file: "keepix_export_list.txt".to_string(),
                    status: format!("Saved list to {}", list_path.display()),
                },
            )
            .ok();
        }

        app.emit(
            "export-progress",
            ExportProgressEvent {
                phase: "complete".to_string(),
                current: total,
                total,
                current_file: "Done!".to_string(),
                status: "Export completed successfully!".to_string(),
            },
        )
        .ok();
    });

    Ok(total)
}

use crate::cache;

// ============================================================================
// Image cache commands
// ============================================================================

/// Read an image file and return it as a base64-encoded data URI.
/// Uses the in-memory LRU cache for high performance.
#[tauri::command]
pub fn read_image_base64(
    cache_state: tauri::State<'_, cache::CacheState>,
    path: String,
) -> Result<String, String> {
    cache::read_image_as_data_uri(&cache_state, &path)
}

/// Batch-preload images into the LRU cache for faster future access.
/// Returns the number of images successfully cached.
#[tauri::command]
pub fn preload_images(
    cache_state: tauri::State<'_, cache::CacheState>,
    paths: Vec<String>,
) -> Result<usize, String> {
    Ok(cache::preload_batch(&cache_state, &paths))
}

/// Clear the entire image cache (useful when switching projects).
#[tauri::command]
pub fn clear_image_cache(
    cache_state: tauri::State<'_, cache::CacheState>,
) -> Result<(), String> {
    cache::clear(&cache_state);
    Ok(())
}

// ============================================================================
// Settings & Adjustments commands
// ============================================================================

#[tauri::command]
pub fn save_adjustments(
    db_state: tauri::State<'_, db::DbState>,
    media_id: String,
    adjustments_json: Option<String>,
) -> Result<(), String> {
    let conn = db_state.conn.lock().map_err(|e| e.to_string())?;
    db::update_adjustments(&conn, &media_id, adjustments_json.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_applied_preset(
    db_state: tauri::State<'_, db::DbState>,
    media_id: String,
    preset_name: Option<String>,
) -> Result<(), String> {
    let conn = db_state.conn.lock().map_err(|e| e.to_string())?;
    db::update_applied_preset(&conn, &media_id, preset_name.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_setting(
    db_state: tauri::State<'_, db::DbState>,
    key: String,
) -> Result<Option<String>, String> {
    let conn = db_state.conn.lock().map_err(|e| e.to_string())?;
    db::get_setting(&conn, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(
    db_state: tauri::State<'_, db::DbState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = db_state.conn.lock().map_err(|e| e.to_string())?;
    db::set_setting(&conn, &key, &value).map_err(|e| e.to_string())
}

// ============================================================================
// Utility commands
// ============================================================================

#[tauri::command]
pub fn convert_file_path(path: String) -> String {
    path.replace('\\', "/")
}

// ============================================================================
// Video metadata commands
// ============================================================================

#[tauri::command]
pub fn save_video_metadata(
    db_state: tauri::State<'_, db::DbState>,
    media_id: String,
    width: i32,
    height: i32,
    duration: f64,
    thumbnail_base64: String,
) -> Result<(), String> {
    use base64::{Engine as _, engine::general_purpose};
    use std::path::Path;
    let conn = db_state.conn.lock().map_err(|e| e.to_string())?;
    
    // Load media item and project
    let item = db::get_media_item(&conn, &media_id).map_err(|e| e.to_string())?;
    let project = db::get_project(&conn, &item.project_id).map_err(|e| e.to_string())?;
    
    // Get thumbnail directory
    let thumb_dir = scanner::get_thumbnail_dir(Path::new(&project.root_path));
    let thumb_path = thumb_dir.join(format!("{}_thumb.webp", media_id));
    let preview_path = thumb_dir.join(format!("{}_preview.webp", media_id));
    
    // Clean and decode base64
    let base64_data = if thumbnail_base64.contains("base64,") {
        thumbnail_base64.split("base64,").nth(1).unwrap_or(&thumbnail_base64)
    } else {
        &thumbnail_base64
    };
    let decoded_bytes = general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("Base64 decode error: {}", e))?;
        
    // Write WebP thumbnail files
    std::fs::write(&thumb_path, &decoded_bytes)
        .map_err(|e| format!("Failed to write thumbnail: {}", e))?;
    std::fs::write(&preview_path, &decoded_bytes)
        .map_err(|e| format!("Failed to write preview: {}", e))?;
        
    // Update thumbnail paths in DB
    db::update_thumbnail(
        &conn,
        &media_id,
        &thumb_path.to_string_lossy(),
        Some(&preview_path.to_string_lossy()),
    )
    .map_err(|e| e.to_string())?;
    
    // Generate and save video EXIF JSON
    let mut exif = metadata::ExifData::default();
    exif.width = Some(width as u32);
    exif.height = Some(height as u32);
    exif.duration = Some(duration);
    exif.file_size = Some(item.file_size);
    exif.file_format = Path::new(&item.file_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_uppercase());
        
    let exif_json = serde_json::to_string(&exif).map_err(|e| e.to_string())?;
    db::update_exif(&conn, &media_id, &exif_json, Some(width), Some(height))
        .map_err(|e| e.to_string())?;
        
    Ok(())
}

#[tauri::command]
pub fn mark_video_unsupported(
    db_state: tauri::State<'_, db::DbState>,
    media_id: String,
) -> Result<(), String> {
    use std::path::Path;
    let conn = db_state.conn.lock().map_err(|e| e.to_string())?;
    let item = db::get_media_item(&conn, &media_id).map_err(|e| e.to_string())?;
    
    let mut exif = metadata::ExifData::default();
    exif.unsupported = Some(true);
    exif.file_size = Some(item.file_size);
    exif.file_format = Path::new(&item.file_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_uppercase());
        
    let exif_json = serde_json::to_string(&exif).map_err(|e| e.to_string())?;
    db::update_exif(&conn, &media_id, &exif_json, Some(-1), Some(-1))
        .map_err(|e| e.to_string())?;
        
    Ok(())
}

fn extract_xmp_from_file(path: &Path) -> Option<String> {
    // 1. If it's an xmp or lrtemplate file itself
    if path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("xmp") || ext.eq_ignore_ascii_case("lrtemplate")) {
        return std::fs::read_to_string(path).ok();
    }

    // 2. Look for sidecar xmp
    let sidecar_path = path.with_extension("xmp");
    if sidecar_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&sidecar_path) {
            return Some(content);
        }
    }

    // 3. Look for embedded XMP in the RAW / photo file itself
    if let Ok(bytes) = std::fs::read(path) {
        let start_pattern = b"<x:xmpmeta";
        let end_pattern = b"</x:xmpmeta>";

        if let Some(start_idx) = bytes.windows(start_pattern.len()).position(|window| window == start_pattern) {
            if let Some(end_idx) = bytes.windows(end_pattern.len()).position(|window| window == end_pattern) {
                if start_idx < end_idx {
                    if let Ok(xmp_str) = String::from_utf8(bytes[start_idx..end_idx + end_pattern.len()].to_vec()) {
                        return Some(xmp_str);
                    }
                }
            }
        }
    }

    None
}

#[tauri::command]
pub fn extract_xmp_preset(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    match extract_xmp_from_file(path) {
        Some(xmp) => Ok(xmp),
        None => Err("No Lightroom preset or XMP metadata found in this file".to_string()),
    }
}

