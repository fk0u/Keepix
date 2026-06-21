use keepix_core::db::{self, DbState, MediaItem};
use keepix_core::metadata;
use keepix_core::scanner;
use keepix_core::thumbnail;
use keepix_core::ai;
use serde::Serialize;
use std::path::Path;
use tauri::{AppHandle, Emitter, Manager};
use rusqlite::params;

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
            ai_aesthetic_score: None,
            ai_technical_score: None,
            ai_overall_score: None,
            ai_confidence: None,
            ai_face_count: None,
            ai_blink_detected: None,
            ai_is_duplicate: Some(0),
            ai_duplicate_group_id: None,
            ai_reasoning: None,
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

use keepix_core::cache;

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

// ============================================================================
// AI Culling & Style commands
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct AiProgressEvent {
    pub current: usize,
    pub total: usize,
    pub phase: String, // "loading" | "scoring" | "duplicates" | "complete"
    pub current_file: String,
    pub aesthetic_score: f64,
    pub technical_score: f64,
    pub overall_score: f64,
    pub face_count: i32,
    pub blink_detected: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DuplicateGroup {
    pub group_id: String,
    pub items: Vec<MediaItem>,
}

#[tauri::command]
pub fn check_ai_models_status(app: AppHandle) -> Result<bool, String> {
    let models_dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("models");
    Ok(ai::ModelManager::check_models_present(&models_dir))
}

#[tauri::command]
pub fn download_ai_models(app: AppHandle) -> Result<(), String> {
    let models_dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("models");
    let app_clone = app.clone();
    std::thread::spawn(move || {
        let progress_fn = move |progress: ai::DownloadProgressEvent| {
            app_clone.emit("download-progress", progress).ok();
        };
        if let Err(e) = ai::ModelManager::download_all_models(&models_dir, progress_fn) {
            log::error!("Failed to download models: {}", e);
            app.emit("download-complete", Err::<(), String>(e)).ok();
        } else {
            app.emit("download-complete", Ok::<(), String>(())).ok();
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn run_local_ai_cull(
    app: AppHandle,
    _state: tauri::State<'_, DbState>,
    project_id: String,
    aesthetic_weight: f64,
    technical_weight: f64,
    face_weight: f64,
    duplicate_threshold: f64,
) -> Result<(), String> {
    let app_clone = app.clone();
    
    // Create new background connection to SQLite database
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let db_state = DbState {
        conn: std::sync::Mutex::new(
            rusqlite::Connection::open(app_data_dir.join("keepix.db")).map_err(|e| e.to_string())?
        ),
    };

    std::thread::spawn(move || {
        // Step 1: Load ONNX Sessions
        app_clone.emit("ai-progress", AiProgressEvent {
            current: 0,
            total: 0,
            phase: "loading".to_string(),
            current_file: "".to_string(),
            aesthetic_score: 0.0,
            technical_score: 0.0,
            overall_score: 0.0,
            face_count: 0,
            blink_detected: false,
            message: "Loading ONNX Models into memory...".to_string(),
        }).ok();

        let models_dir = app_data_dir.join("models");
        let mut manager = ai::ModelManager::new();
        if let Err(e) = manager.load_sessions(&models_dir) {
            log::error!("Failed to load ONNX sessions: {}", e);
            app_clone.emit("ai-progress", AiProgressEvent {
                current: 0,
                total: 0,
                phase: "complete".to_string(),
                current_file: "".to_string(),
                aesthetic_score: 0.0,
                technical_score: 0.0,
                overall_score: 0.0,
                face_count: 0,
                blink_detected: false,
                message: format!("Error loading models: {}", e),
            }).ok();
            return;
        }

        let clip_sess = manager.clip_session.as_mut().unwrap();
        let aes_sess = manager.aesthetic_session.as_mut().unwrap();
        let musiq_sess = manager.musiq_session.as_mut().unwrap();
        let face_sess = manager.face_session.as_mut().unwrap();

        // Step 2: Query uncategorized photos for this project
        let photos = {
            if let Ok(conn) = db_state.conn.lock() {
                // Get all photos in project (uncategorized only)
                db::get_media_items(
                    &conn,
                    &project_id,
                    None,
                    None,
                    true, // uncategorized only
                    None,
                    None,
                    None,
                    "name",
                    "asc",
                    0,
                    100000,
                ).unwrap_or(Vec::new())
            } else {
                Vec::new()
            }
        };

        let total = photos.len();
        if total == 0 {
            app_clone.emit("ai-progress", AiProgressEvent {
                current: 0,
                total: 0,
                phase: "complete".to_string(),
                current_file: "".to_string(),
                aesthetic_score: 0.0,
                technical_score: 0.0,
                overall_score: 0.0,
                face_count: 0,
                blink_detected: false,
                message: "No photos found to process.".to_string(),
            }).ok();
            return;
        }

        // Step 3: Run pipeline for each photo
        for (i, photo) in photos.iter().enumerate() {
            let path = Path::new(&photo.file_path);
            if !path.exists() {
                continue;
            }

            // Preprocess image tensor once for CLIP & Technical scorers
            let tensor = match ai::preprocess_for_clip(path) {
                Ok(t) => t,
                Err(e) => {
                    log::warn!("Preprocessing failed for {}: {}", photo.file_name, e);
                    continue;
                }
            };

            // A. Run CLIP & Save Embedding
            let embedding = match ai::get_clip_embedding(clip_sess, &tensor) {
                Ok(emb) => {
                    if let Ok(conn) = db_state.conn.lock() {
                        db::insert_embedding(&conn, &photo.id, &emb).ok();
                    }
                    emb
                }
                Err(e) => {
                    log::warn!("CLIP error for {}: {}", photo.file_name, e);
                    continue;
                }
            };

            // B. Run Aesthetic Scorer (using CLIP embedding slice)
            let aesthetic = ai::get_aesthetic_score(aes_sess, &embedding).unwrap_or(5.0);

            // C. Run Technical Quality Scorer (using the preprocessed image tensor)
            let technical = ai::get_technical_score(musiq_sess, &tensor).unwrap_or(50.0);

            // D. Run Face Detector (does its own resizing inside detect_faces_and_blinks)
            let (face_count, blink_detected) = ai::detect_faces_and_blinks(face_sess, path).unwrap_or((0, 0));

            // E. Overall Score and Confidence Computation
            // Standardize scores to [0.0, 1.0] range
            let norm_aesthetic = (aesthetic as f64 / 10.0).clamp(0.0, 1.0);
            let norm_technical = (technical as f64 / 100.0).clamp(0.0, 1.0);

            // Compose weights
            let mut overall = (norm_aesthetic * aesthetic_weight) + (norm_technical * technical_weight);
            
            // Apply Blink and Face Penalty if eyes are closed on a detected face
            if face_count > 0 {
                if blink_detected == 1 {
                    overall -= face_weight * 0.5; // subtract penalty
                } else {
                    overall += face_weight * 0.2; // positive for good open eyes
                }
            }
            let overall_score = overall.clamp(0.0, 1.0);
            let confidence = 0.85; // Standard ONNX confidence baseline

            // F. Create reasoning text
            let reasoning = format!(
                "Aesthetic: {:.1}/10, Sharpness: {:.0}/100, Faces: {}, Blink: {}",
                aesthetic,
                technical,
                face_count,
                if blink_detected == 1 { "YES" } else { "NO" }
            );

            // G. Save back to database
            if let Ok(conn) = db_state.conn.lock() {
                db::update_media_ai_scores(
                    &conn,
                    &photo.id,
                    Some(aesthetic as f64),
                    Some(technical as f64),
                    Some(overall_score),
                    Some(confidence),
                    Some(face_count),
                    Some(blink_detected),
                    Some(0), // is_duplicate (0 initially)
                    None, // duplicate_group_id
                    Some(&reasoning),
                ).ok();

                // If overall score is very high (> 0.70), auto-recommend BEST. If very low (< 0.25), TRASH.
                // Otherwise, leave uncategorized or review.
                let cat_id = if overall_score > 0.75 {
                    Some(2) // Simpan / Best
                } else if overall_score < 0.25 {
                    Some(1) // Buang / Trash
                } else {
                    None // Uncategorized / Review
                };
                if cat_id.is_some() {
                    db::set_media_category(&conn, &photo.id, cat_id).ok();
                }
            }

            // Emit Progress
            app_clone.emit("ai-progress", AiProgressEvent {
                current: i + 1,
                total,
                phase: "scoring".to_string(),
                current_file: photo.file_name.clone(),
                aesthetic_score: aesthetic as f64,
                technical_score: technical as f64,
                overall_score,
                face_count,
                blink_detected: blink_detected == 1,
                message: format!("Scored {} successfully", photo.file_name),
            }).ok();
        }

        // Step 4: Run Duplicate & Similarity detection across all project embeddings
        app_clone.emit("ai-progress", AiProgressEvent {
            current: total,
            total,
            phase: "duplicates".to_string(),
            current_file: "".to_string(),
            aesthetic_score: 0.0,
            technical_score: 0.0,
            overall_score: 0.0,
            face_count: 0,
            blink_detected: false,
            message: "Running duplicate & similarity checks...".to_string(),
        }).ok();

        if let Ok(conn) = db_state.conn.lock() {
            // Retrieve all items & embeddings in project
            let all_items = db::get_media_items(
                &conn,
                &project_id,
                None,
                None,
                false, // include all
                None,
                None,
                None,
                "name",
                "asc",
                0,
                100000,
            ).unwrap_or(Vec::new());

            let mut embeddings = Vec::new();
            for item in &all_items {
                if let Ok(Some(emb)) = db::get_embedding(&conn, &item.id) {
                    embeddings.push((item.id.clone(), item.ai_overall_score.unwrap_or(0.0), emb));
                }
            }

            let mut visited = std::collections::HashSet::new();
            for (id1, score1, emb1) in &embeddings {
                if visited.contains(id1) {
                    continue;
                }

                let mut group = vec![(id1.clone(), *score1)];
                for (id2, score2, emb2) in &embeddings {
                    if id1 == id2 || visited.contains(id2) {
                        continue;
                    }

                    let similarity = ai::cosine_similarity(emb1, emb2);
                    if similarity > duplicate_threshold as f32 {
                        group.push((id2.clone(), *score2));
                    }
                }

                if group.len() > 1 {
                    let group_id = uuid::Uuid::new_v4().to_string();
                    
                    // Sort items by score descending
                    group.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                    
                    // Mark the highest-scoring as primary, others as duplicates
                    for (idx, (media_id, _)) in group.iter().enumerate() {
                        let is_duplicate = if idx == 0 { 0 } else { 1 };
                        
                        // If it is marked duplicate, we can auto-category it to Trash (1) or color label it!
                        if is_duplicate == 1 {
                            db::set_media_category(&conn, media_id, Some(1)).ok(); // Auto-Trash
                            db::set_color_label(&conn, media_id, Some("red")).ok(); // Mark with red label
                        }
                        
                        conn.execute(
                            "UPDATE media_items SET 
                                ai_is_duplicate = ?1, 
                                ai_duplicate_group_id = ?2,
                                ai_reasoning = ai_reasoning || ?3
                             WHERE id = ?4",
                            params![
                                is_duplicate,
                                group_id,
                                if is_duplicate == 1 { " | Duplicate (Discard recommendation)" } else { " | Duplicate Primary (Keep recommendation)" },
                                media_id
                            ],
                        ).ok();

                        visited.insert(media_id.clone());
                    }
                }
            }
        }

        // Step 5: Final completion
        app_clone.emit("ai-progress", AiProgressEvent {
            current: total,
            total,
            phase: "complete".to_string(),
            current_file: "".to_string(),
            aesthetic_score: 0.0,
            technical_score: 0.0,
            overall_score: 0.0,
            face_count: 0,
            blink_detected: false,
            message: "Local AI Culling workflow complete!".to_string(),
        }).ok();
    });

    Ok(())
}

#[tauri::command]
pub fn train_and_predict_style(
    state: tauri::State<'_, DbState>,
    project_id: String,
) -> Result<usize, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;

    // 1. Get user manual culls (Best / Trash labeled CLIP embeddings)
    let labeled_data = db::get_labeled_embeddings(&conn, &project_id).map_err(|e| e.to_string())?;
    if labeled_data.is_empty() {
        return Ok(0); // No dataset to learn style from
    }

    // 2. Get all uncategorized photos that have CLIP embeddings
    let uncategorized = db::get_media_items(
        &conn,
        &project_id,
        None,
        None,
        true, // uncategorized only
        None,
        None,
        None,
        "name",
        "asc",
        0,
        100000,
    ).map_err(|e| e.to_string())?;

    let mut trained_count = 0;
    for photo in &uncategorized {
        if let Ok(Some(embedding)) = db::get_embedding(&conn, &photo.id) {
            // Predict style using KNN classification
            let (predicted_cat, confidence) = ai::predict_user_style_knn(&embedding, &labeled_data, 5);
            
            let reasoning = format!(
                "Learn My Style recommendation: {} (Confidence: {:.0}%)",
                if predicted_cat == 2 { "KEEP" } else { "DISCARD" },
                confidence * 100.0
            );

            // Non-destructively write recommendation to reasoning & confidence, and set category suggestion
            conn.execute(
                "UPDATE media_items SET 
                    ai_confidence = ?1,
                    ai_reasoning = ?2
                 WHERE id = ?3",
                params![confidence, reasoning, photo.id],
            ).ok();

            // Set color label: green for predicted KEEP, red for predicted DISCARD
            if confidence > 0.80 {
                let color = if predicted_cat == 2 { "green" } else { "red" };
                db::set_color_label(&conn, &photo.id, Some(color)).ok();
            }

            trained_count += 1;
        }
    }

    Ok(trained_count)
}

#[tauri::command]
pub fn query_duplicate_groups(
    state: tauri::State<'_, DbState>,
    project_id: String,
) -> Result<Vec<DuplicateGroup>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;

    // Query distinct group IDs
    let mut stmt = conn.prepare(
        "SELECT DISTINCT ai_duplicate_group_id 
         FROM media_items 
         WHERE project_id = ?1 AND ai_duplicate_group_id IS NOT NULL",
    ).map_err(|e| e.to_string())?;

    let group_ids = stmt.query_map(params![project_id], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| e.to_string())?;

    let mut duplicate_groups = Vec::new();
    for group_id in group_ids {
        // Query items for this duplicate group
        let mut item_stmt = conn.prepare(
            "SELECT id, project_id, file_path, file_name, file_type, file_size,
                    width, height, category_id, star_rating, color_label,
                    thumbnail_path, preview_path, exif_json, file_hash, date_taken,
                    adjustments, applied_preset, created_at, updated_at,
                    ai_aesthetic_score, ai_technical_score, ai_overall_score,
                    ai_confidence, ai_face_count, ai_blink_detected,
                    ai_is_duplicate, ai_duplicate_group_id, ai_reasoning
             FROM media_items 
             WHERE ai_duplicate_group_id = ?1
             ORDER BY ai_overall_score DESC",
        ).map_err(|e| e.to_string())?;

        let items = item_stmt.query_map(params![group_id], |row| {
            Ok(MediaItem {
                id: row.get(0)?,
                project_id: row.get(1)?,
                file_path: row.get(2)?,
                file_name: row.get(3)?,
                file_type: row.get(4)?,
                file_size: row.get(5)?,
                width: row.get(6)?,
                height: row.get(7)?,
                category_id: row.get(8)?,
                star_rating: row.get(9)?,
                color_label: row.get(10)?,
                thumbnail_path: row.get(11)?,
                preview_path: row.get(12)?,
                exif_json: row.get(13)?,
                file_hash: row.get(14)?,
                date_taken: row.get(15)?,
                adjustments: row.get(16)?,
                applied_preset: row.get(17)?,
                created_at: row.get(18)?,
                updated_at: row.get(19)?,
                ai_aesthetic_score: row.get(20)?,
                ai_technical_score: row.get(21)?,
                ai_overall_score: row.get(22)?,
                ai_confidence: row.get(23)?,
                ai_face_count: row.get(24)?,
                ai_blink_detected: row.get(25)?,
                ai_is_duplicate: row.get(26)?,
                ai_duplicate_group_id: row.get(27)?,
                ai_reasoning: row.get(28)?,
            })
        }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<MediaItem>, _>>()
        .map_err(|e| e.to_string())?;

        duplicate_groups.push(DuplicateGroup { group_id, items });
    }

    Ok(duplicate_groups)
}

#[tauri::command]
pub async fn query_ollama_vision(
    image_base64: String,
    model: String,
    prompt: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    // Construct request payload matching Ollama generate schema
    let payload = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "images": [image_base64]
    });

    let res = client.post("http://localhost:11434/api/generate")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to local Ollama service: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Ollama API error: Status {}", res.status()));
    }

    let data = res.json::<serde_json::Value>().await
        .map_err(|e| format!("Failed to parse response from Ollama: {}", e))?;

    let response_text = data["response"].as_str()
        .ok_or_else(|| "Ollama response did not contain 'response' field".to_string())?;

    Ok(response_text.to_string())
}

#[tauri::command]
pub async fn query_nvidia_nim_vision(
    image_base64: String,
    model: String,
    prompt: String,
    api_key: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let image_url = if image_base64.starts_with("data:") {
        image_base64
    } else {
        format!("data:image/jpeg;base64,{}", image_base64)
    };

    let payload = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": prompt
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": image_url
                        }
                    }
                ]
            }
        ],
        "max_tokens": 512,
        "temperature": 0.2
    });

    let res = client.post("https://integrate.api.nvidia.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to NVIDIA NIM service: {}", e))?;

    if !res.status().is_success() {
        let status = res.status();
        let err_body = res.text().await.unwrap_or_default();
        return Err(format!("NVIDIA NIM API error: Status {}. Details: {}", status, err_body));
    }

    let data = res.json::<serde_json::Value>().await
        .map_err(|e| format!("Failed to parse response from NVIDIA NIM: {}", e))?;

    let response_text = data["choices"][0]["message"]["content"].as_str()
        .ok_or_else(|| "NVIDIA NIM response did not contain message content".to_string())?;

    Ok(response_text.to_string())
}


