use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

/// Database state managed by Tauri
pub struct DbState {
    pub conn: Mutex<Connection>,
}

/// Represents a project (a culling session)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub created_at: String,
    pub last_opened: String,
    pub total_items: i64,
}

/// Represents a media item (photo or video)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: String,
    pub project_id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_type: String, // "photo" or "video"
    pub file_size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub category_id: Option<i32>,
    pub star_rating: i32,
    pub color_label: Option<String>,
    pub thumbnail_path: Option<String>,
    pub preview_path: Option<String>,
    pub exif_json: Option<String>,
    pub file_hash: Option<String>,
    pub date_taken: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Category definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub color: String,
    pub icon: String,
    pub hotkey: String,
    pub sort_order: i32,
}

/// Undo history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoEntry {
    pub id: i64,
    pub project_id: String,
    pub media_id: String,
    pub action_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub created_at: String,
}

/// Statistics per category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category_id: Option<i32>,
    pub category_name: String,
    pub count: i64,
}

/// Initialize the database connection and run migrations
pub fn init_db(app_data_dir: &PathBuf) -> Result<Connection> {
    std::fs::create_dir_all(app_data_dir).ok();
    let db_path = app_data_dir.join("keepix.db");
    let conn = Connection::open(db_path)?;

    // Enable WAL mode for better concurrent read performance
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA synchronous=NORMAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    run_migrations(&conn)?;
    Ok(conn)
}

/// Run database migrations
fn run_migrations(conn: &Connection) -> Result<()> {
    let user_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if user_version < 1 {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                root_path TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_opened TEXT NOT NULL DEFAULT (datetime('now')),
                total_items INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                icon TEXT NOT NULL DEFAULT '',
                hotkey TEXT NOT NULL DEFAULT '',
                sort_order INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS media_items (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_type TEXT NOT NULL DEFAULT 'photo',
                file_size INTEGER NOT NULL DEFAULT 0,
                width INTEGER,
                height INTEGER,
                category_id INTEGER,
                star_rating INTEGER NOT NULL DEFAULT 0,
                color_label TEXT,
                thumbnail_path TEXT,
                preview_path TEXT,
                exif_json TEXT,
                file_hash TEXT,
                date_taken TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
                FOREIGN KEY (category_id) REFERENCES categories(id)
            );

            CREATE INDEX IF NOT EXISTS idx_media_project ON media_items(project_id);
            CREATE INDEX IF NOT EXISTS idx_media_category ON media_items(category_id);
            CREATE INDEX IF NOT EXISTS idx_media_file_path ON media_items(file_path);
            CREATE INDEX IF NOT EXISTS idx_media_file_hash ON media_items(file_hash);
            CREATE INDEX IF NOT EXISTS idx_media_star_rating ON media_items(star_rating);

            CREATE TABLE IF NOT EXISTS undo_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL,
                media_id TEXT NOT NULL,
                action_type TEXT NOT NULL,
                old_value TEXT,
                new_value TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            -- Insert default categories
            INSERT OR IGNORE INTO categories (id, name, color, icon, hotkey, sort_order) VALUES
                (1, 'Buang', '#ef4444', 'trash', '1', 1),
                (2, 'Simpan', '#22c55e', 'star', '2', 2),
                (3, 'Simpan Draft', '#f59e0b', 'bookmark', '3', 3),
                (4, 'Review', '#6366f1', 'eye', '4', 4);

            PRAGMA user_version = 1;
            ",
        )?;
    }

    Ok(())
}

// ============================================================================
// Project operations
// ============================================================================

/// Create a new project
pub fn create_project(conn: &Connection, id: &str, name: &str, root_path: &str) -> Result<Project> {
    conn.execute(
        "INSERT INTO projects (id, name, root_path) VALUES (?1, ?2, ?3)",
        params![id, name, root_path],
    )?;

    get_project(conn, id)
}

/// Get a project by ID
pub fn get_project(conn: &Connection, id: &str) -> Result<Project> {
    conn.query_row(
        "SELECT id, name, root_path, created_at, last_opened, total_items FROM projects WHERE id = ?1",
        params![id],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                created_at: row.get(3)?,
                last_opened: row.get(4)?,
                total_items: row.get(5)?,
            })
        },
    )
}

/// List all projects ordered by last opened
pub fn list_projects(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, root_path, created_at, last_opened, total_items FROM projects ORDER BY last_opened DESC",
    )?;

    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                created_at: row.get(3)?,
                last_opened: row.get(4)?,
                total_items: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(projects)
}

/// Update project's last opened timestamp and total item count
pub fn update_project_stats(conn: &Connection, project_id: &str) -> Result<()> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM media_items WHERE project_id = ?1",
        params![project_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "UPDATE projects SET last_opened = datetime('now'), total_items = ?1 WHERE id = ?2",
        params![count, project_id],
    )?;

    Ok(())
}

/// Delete a project and all its media items
pub fn delete_project(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM undo_history WHERE project_id = ?1", params![id])?;
    conn.execute("DELETE FROM media_items WHERE project_id = ?1", params![id])?;
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}

// ============================================================================
// Media item operations
// ============================================================================

/// Insert a batch of media items
pub fn insert_media_batch(conn: &Connection, items: &[MediaItem]) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    {
        let mut stmt = tx.prepare(
            "INSERT OR IGNORE INTO media_items (id, project_id, file_path, file_name, file_type, file_size, width, height, date_taken)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )?;

        for item in items {
            stmt.execute(params![
                item.id,
                item.project_id,
                item.file_path,
                item.file_name,
                item.file_type,
                item.file_size,
                item.width,
                item.height,
                item.date_taken,
            ])?;
        }
    }

    tx.commit()?;
    Ok(())
}

/// Get paginated media items for a project with optional filters
pub fn get_media_items(
    conn: &Connection,
    project_id: &str,
    category_filter: Option<i32>,
    star_filter: Option<i32>,
    uncategorized_only: bool,
    camera_model: Option<&str>,
    lens_model: Option<&str>,
    color_label: Option<&str>,
    sort_by: &str,
    sort_order: &str,
    offset: i64,
    limit: i64,
) -> Result<Vec<MediaItem>> {
    let order_col = match sort_by {
        "date" => "date_taken",
        "size" => "file_size",
        "rating" => "star_rating",
        "type" => "file_type",
        _ => "file_name",
    };

    let order_dir = match sort_order {
        "desc" => "DESC",
        _ => "ASC",
    };

    let sql = format!(
        "SELECT id, project_id, file_path, file_name, file_type, file_size,
                width, height, category_id, star_rating, color_label,
                thumbnail_path, preview_path, exif_json, file_hash, date_taken,
                created_at, updated_at
         FROM media_items 
         WHERE project_id = :project_id
           AND (:category_filter IS NULL OR category_id = :category_filter)
           AND (:uncategorized_only = 0 OR category_id IS NULL)
           AND (:star_filter IS NULL OR star_rating >= :star_filter)
           AND (:camera_model IS NULL OR json_extract(exif_json, '$.camera_model') = :camera_model)
           AND (:lens_model IS NULL OR json_extract(exif_json, '$.lens_model') = :lens_model)
           AND (:color_label IS NULL OR color_label = :color_label)
         ORDER BY {} {} 
         LIMIT :limit OFFSET :offset",
        order_col, order_dir
    );

    let mut stmt = conn.prepare(&sql)?;
    let items = stmt
        .query_map(
            rusqlite::named_params! {
                ":project_id": project_id,
                ":category_filter": category_filter,
                ":uncategorized_only": if uncategorized_only { 1 } else { 0 },
                ":star_filter": star_filter,
                ":camera_model": camera_model,
                ":lens_model": lens_model,
                ":color_label": color_label,
                ":limit": limit,
                ":offset": offset,
            },
            |row| {
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
                    created_at: row.get(16)?,
                    updated_at: row.get(17)?,
                })
            },
        )?
        .collect::<Result<Vec<_>>>()?;

    Ok(items)
}

/// Set the category for a media item (with undo support)
pub fn set_media_category(
    conn: &Connection,
    media_id: &str,
    category_id: Option<i32>,
) -> Result<Option<i32>> {
    // Get the old category for undo
    let old_category: Option<i32> = conn
        .query_row(
            "SELECT category_id FROM media_items WHERE id = ?1",
            params![media_id],
            |row| row.get(0),
        )
        .ok();

    conn.execute(
        "UPDATE media_items SET category_id = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![category_id, media_id],
    )?;

    Ok(old_category)
}

/// Set star rating for a media item
pub fn set_star_rating(conn: &Connection, media_id: &str, rating: i32) -> Result<()> {
    conn.execute(
        "UPDATE media_items SET star_rating = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![rating, media_id],
    )?;
    Ok(())
}

/// Set color label for a media item
pub fn set_color_label(conn: &Connection, media_id: &str, color: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE media_items SET color_label = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![color, media_id],
    )?;
    Ok(())
}

/// Update thumbnail path for a media item
pub fn update_thumbnail(
    conn: &Connection,
    media_id: &str,
    thumbnail_path: &str,
    preview_path: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE media_items SET thumbnail_path = ?1, preview_path = ?2 WHERE id = ?3",
        params![thumbnail_path, preview_path, media_id],
    )?;
    Ok(())
}

/// Update EXIF metadata for a media item
pub fn update_exif(
    conn: &Connection,
    media_id: &str,
    exif_json: &str,
    width: Option<i32>,
    height: Option<i32>,
) -> Result<()> {
    conn.execute(
        "UPDATE media_items SET exif_json = ?1, width = ?2, height = ?3 WHERE id = ?4",
        params![exif_json, width, height, media_id],
    )?;
    Ok(())
}

/// Get category statistics for a project
pub fn get_category_stats(conn: &Connection, project_id: &str) -> Result<Vec<CategoryStats>> {
    let mut stats = Vec::new();

    // Count uncategorized
    let uncategorized: i64 = conn.query_row(
        "SELECT COUNT(*) FROM media_items WHERE project_id = ?1 AND category_id IS NULL",
        params![project_id],
        |row| row.get(0),
    )?;
    stats.push(CategoryStats {
        category_id: None,
        category_name: "Uncategorized".to_string(),
        count: uncategorized,
    });

    // Count per category
    let mut stmt = conn.prepare(
        "SELECT c.id, c.name, COUNT(m.id) as cnt
         FROM categories c
         LEFT JOIN media_items m ON m.category_id = c.id AND m.project_id = ?1
         GROUP BY c.id
         ORDER BY c.sort_order",
    )?;

    let cat_stats = stmt
        .query_map(params![project_id], |row| {
            Ok(CategoryStats {
                category_id: Some(row.get(0)?),
                category_name: row.get(1)?,
                count: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    stats.extend(cat_stats);
    Ok(stats)
}

/// Get all categories
pub fn get_categories(conn: &Connection) -> Result<Vec<Category>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, icon, hotkey, sort_order FROM categories ORDER BY sort_order",
    )?;

    let categories = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                icon: row.get(3)?,
                hotkey: row.get(4)?,
                sort_order: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(categories)
}

// ============================================================================
// Undo operations
// ============================================================================

/// Push an undo entry
pub fn push_undo(
    conn: &Connection,
    project_id: &str,
    media_id: &str,
    action_type: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO undo_history (project_id, media_id, action_type, old_value, new_value)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![project_id, media_id, action_type, old_value, new_value],
    )?;
    Ok(())
}

/// Pop the last undo entry for a project
pub fn pop_undo(conn: &Connection, project_id: &str) -> Result<Option<UndoEntry>> {
    let entry = conn
        .query_row(
            "SELECT id, project_id, media_id, action_type, old_value, new_value, created_at
             FROM undo_history WHERE project_id = ?1 ORDER BY id DESC LIMIT 1",
            params![project_id],
            |row| {
                Ok(UndoEntry {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    media_id: row.get(2)?,
                    action_type: row.get(3)?,
                    old_value: row.get(4)?,
                    new_value: row.get(5)?,
                    created_at: row.get(6)?,
                })
            },
        )
        .ok();

    if let Some(ref e) = entry {
        conn.execute("DELETE FROM undo_history WHERE id = ?1", params![e.id])?;
    }

    Ok(entry)
}

/// Get a single media item by ID
pub fn get_media_item(conn: &Connection, media_id: &str) -> Result<MediaItem> {
    conn.query_row(
        "SELECT id, project_id, file_path, file_name, file_type, file_size,
                width, height, category_id, star_rating, color_label,
                thumbnail_path, preview_path, exif_json, file_hash, date_taken,
                created_at, updated_at
         FROM media_items WHERE id = ?1",
        params![media_id],
        |row| {
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
                created_at: row.get(16)?,
                updated_at: row.get(17)?,
            })
        },
    )
}

/// Get total count of media items for a project (with filters)
pub fn get_media_count(
    conn: &Connection,
    project_id: &str,
    category_filter: Option<i32>,
    star_filter: Option<i32>,
    uncategorized_only: bool,
    camera_model: Option<&str>,
    lens_model: Option<&str>,
    color_label: Option<&str>,
) -> Result<i64> {
    let sql = "SELECT COUNT(*) FROM media_items 
               WHERE project_id = :project_id
                 AND (:category_filter IS NULL OR category_id = :category_filter)
                 AND (:uncategorized_only = 0 OR category_id IS NULL)
                 AND (:star_filter IS NULL OR star_rating >= :star_filter)
                 AND (:camera_model IS NULL OR json_extract(exif_json, '$.camera_model') = :camera_model)
                 AND (:lens_model IS NULL OR json_extract(exif_json, '$.lens_model') = :lens_model)
                 AND (:color_label IS NULL OR color_label = :color_label)";

    conn.query_row(
        sql,
        rusqlite::named_params! {
            ":project_id": project_id,
            ":category_filter": category_filter,
            ":uncategorized_only": if uncategorized_only { 1 } else { 0 },
            ":star_filter": star_filter,
            ":camera_model": camera_model,
            ":lens_model": lens_model,
            ":color_label": color_label,
        },
        |row| row.get(0),
    )
}

/// Unique EXIF filter options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExifFilters {
    pub camera_models: Vec<String>,
    pub lens_models: Vec<String>,
    pub color_labels: Vec<String>,
}

/// Query distinct camera/lens models and color labels from database
pub fn get_unique_exif_metadata(conn: &Connection, project_id: &str) -> Result<ExifFilters> {
    let mut camera_models = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT json_extract(exif_json, '$.camera_model') 
             FROM media_items 
             WHERE project_id = ?1 AND exif_json IS NOT NULL",
        )?;
        let mut rows = stmt.query(params![project_id])?;
        while let Some(row) = rows.next()? {
            if let Some(val) = row.get::<_, Option<String>>(0)? {
                let trimmed = val.trim().to_string();
                if !trimmed.is_empty() && !camera_models.contains(&trimmed) {
                    camera_models.push(trimmed);
                }
            }
        }
    }

    let mut lens_models = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT json_extract(exif_json, '$.lens_model') 
             FROM media_items 
             WHERE project_id = ?1 AND exif_json IS NOT NULL",
        )?;
        let mut rows = stmt.query(params![project_id])?;
        while let Some(row) = rows.next()? {
            if let Some(val) = row.get::<_, Option<String>>(0)? {
                let trimmed = val.trim().to_string();
                if !trimmed.is_empty() && !lens_models.contains(&trimmed) {
                    lens_models.push(trimmed);
                }
            }
        }
    }

    let mut color_labels = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT color_label 
             FROM media_items 
             WHERE project_id = ?1 AND color_label IS NOT NULL",
        )?;
        let mut rows = stmt.query(params![project_id])?;
        while let Some(row) = rows.next()? {
            if let Some(val) = row.get::<_, Option<String>>(0)? {
                let trimmed = val.trim().to_string();
                if !trimmed.is_empty() && !color_labels.contains(&trimmed) {
                    color_labels.push(trimmed);
                }
            }
        }
    }

    camera_models.sort();
    lens_models.sort();
    color_labels.sort();

    Ok(ExifFilters {
        camera_models,
        lens_models,
        color_labels,
    })
}
