mod commands;

#[cfg(target_env = "msvc")]
#[link(name = "msvcprt")]
extern "C" {}

use keepix_core::db::DbState;
use keepix_core::cache::CacheState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize database
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            let conn = keepix_core::db::init_db(&app_data_dir)
                .expect("Failed to initialize database");

            let cache_limit = keepix_core::db::get_setting(&conn, "cache_limit")
                .unwrap_or(None)
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(300);

            // Store the database connection in Tauri's managed state
            app.manage(DbState {
                conn: Mutex::new(conn),
            });

            // Initialize the image cache
            app.manage(CacheState::new(cache_limit));

            log::info!("Keepix initialized. Data dir: {:?}", app_data_dir);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Project commands
            commands::get_projects,
            commands::create_project,
            commands::delete_project,
            commands::get_project,
            // Scan commands
            commands::scan_folder,
            // Media commands
            commands::get_media_items,
            commands::get_exif_filters,
            commands::get_single_media,
            commands::set_category,
            commands::set_star_rating,
            commands::set_color_label,
            commands::get_metadata,
            // Undo
            commands::undo_last_action,
            // Categories & stats
            commands::get_categories,
            commands::get_category_stats,
            // Batch operations
            commands::batch_set_category,
            commands::export_media_items,
            // Image cache
            commands::read_image_base64,
            commands::preload_images,
            commands::clear_image_cache,
            commands::update_cache_capacity,
            // Settings & Adjustments
            commands::save_adjustments,
            commands::save_applied_preset,
            commands::get_setting,
            commands::set_setting,
            commands::extract_xmp_preset,
            // Utilities
            commands::convert_file_path,
            // Video processing
            commands::save_video_metadata,
            commands::mark_video_unsupported,
            // Local AI Culling & Style Workstation
            commands::check_ai_models_status,
            commands::download_ai_models,
            commands::run_local_ai_cull,
            commands::train_and_predict_style,
            commands::query_duplicate_groups,
            commands::query_ollama_vision,
            commands::query_nvidia_nim_vision,
            commands::query_gemini_vision,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Keepix");
}
