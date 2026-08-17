use rusqlite::{Connection, Result};
use std::fs::create_dir_all;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

pub struct DbState {
    pub conn: std::sync::Mutex<Connection>,
}

#[derive(Debug)]
pub struct DbContent {
    pub file_path: String,
    pub file_name: String,
    pub file_hash: String,
    pub extension: String,
    pub file_size: i64,

    pub inode: Option<i64>,
    pub mime_type: Option<String>,
    pub category: Option<String>,
    pub content_text: Option<String>,
    pub ai_summary: Option<String>,
    pub ai_keywords: Option<String>,

    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub indexed_at: Option<String>,

    pub ai_status: Option<String>,
    pub ai_error: Option<String>,
    pub last_accessed_at: Option<String>,
    pub last_seen_scan_id: Option<i64>,
}

/// Opens the app database, registers sqlite-vec, and applies the schema migration.
pub fn init_database(app_handle: &AppHandle) -> Result<(Connection, PathBuf)> {
    let app_dir = app_handle
        .path()
        .app_config_dir()
        .expect("Error accessing app directory");
    create_dir_all(&app_dir).expect("Failed to create app database folder");

    let db_path = app_dir.join("app.db");

    unsafe {
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite_vec::sqlite3_vec_init as *const (),
        )));
    }

    let conn = Connection::open(&db_path)?;

    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;

    let version: i32 = conn.query_row("PRAGMA user_version;", [], |row| row.get(0))?;

    match version {
        0 => {
            create_schema(&conn)?;

            conn.execute("PRAGMA user_version = 1;", [])?;
        }

        1 => {}

        _ => {
            panic!("Unknown database version: {}", version);
        }
    }

    Ok((conn, db_path))
}

fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        -- COMMAND 1: Core Relational Table
        CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path TEXT NOT NULL UNIQUE,
            file_name TEXT NOT NULL,
            file_hash TEXT NOT NULL,
            inode INTEGER,
            extension TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            mime_type TEXT,
            category TEXT,
            content_text TEXT,
            ai_summary TEXT,
            ai_keywords TEXT,
            created_at DATETIME,
            modified_at DATETIME,
            indexed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            ai_status TEXT DEFAULT 'pending',
            ai_error TEXT,
            last_accessed_at DATETIME,
            last_seen_scan_id INTEGER DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_files_path ON files(file_path);
        CREATE INDEX IF NOT EXISTS idx_files_status ON files(ai_status);

        -- COMMAND 2: Full-Text Search Table & Triggers
        CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
            id UNINDEXED,
            content_text,
            ai_summary,
            ai_keywords,
            content='files',
            content_rowid='rowid'
        );

        -- Trigger: After Inserting a File
        CREATE TRIGGER IF NOT EXISTS files_ai AFTER INSERT ON files BEGIN
            INSERT INTO files_fts(rowid, content_text, ai_summary, ai_keywords) 
            VALUES (new.id, new.content_text, new.ai_summary, new.ai_keywords);
        END;

        -- Trigger: After Deleting a File
        CREATE TRIGGER IF NOT EXISTS files_ad AFTER DELETE ON files BEGIN
            INSERT INTO files_fts(files_fts, rowid, content_text, ai_summary, ai_keywords) 
            VALUES('delete', old.id, old.content_text, old.ai_summary, old.ai_keywords);
        END;

        -- Trigger: After Updating a File (Delete old FTS entry, Insert new)
        CREATE TRIGGER IF NOT EXISTS files_au AFTER UPDATE ON files BEGIN
            INSERT INTO files_fts(files_fts, rowid, content_text, ai_summary, ai_keywords) 
            VALUES('delete', old.id, old.content_text, old.ai_summary, old.ai_keywords);
            
            INSERT INTO files_fts(rowid, content_text, ai_summary, ai_keywords) 
            VALUES (new.id, new.content_text, new.ai_summary, new.ai_keywords);
        END;

        -- COMMAND 3: Vector Search Tables (Enabled by sqlite3_auto_extension above)
        CREATE VIRTUAL TABLE IF NOT EXISTS files_vec USING vec0(
            id INTEGER PRIMARY KEY,
            embedding float[768]
        );

        -- Trigger: Delete embedding when file is removed
        CREATE TRIGGER IF NOT EXISTS files_vec_ad AFTER DELETE ON files BEGIN
            DELETE FROM files_vec WHERE id = old.id;
        END;
        "#,
    )?;
    Ok(())
}
