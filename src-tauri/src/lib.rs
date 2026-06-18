mod cache;
mod commands;
mod db;
mod metadata;
mod scanner;
mod thumbnail;

use db::DbState;
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

            let conn = db::init_db(&app_data_dir)
                .expect("Failed to initialize database");

            // Store the database connection in Tauri's managed state
            app.manage(DbState {
                conn: Mutex::new(conn),
            });

            // Initialize the image cache
            app.manage(cache::CacheState::new());

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
            // Settings & Adjustments
            commands::save_adjustments,
            commands::save_applied_preset,
            commands::get_setting,
            commands::set_setting,
            // Utilities
            commands::convert_file_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Keepix");
}
