use rusqlite::{Connection, Result};
use std::fs::create_dir_all;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

pub struct DbState {
    pub conn: std::sync::Mutex<Connection>,
}

/// Initializes the user database path, loads vector support, and structures tables.
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

        1 => {
            // Database is already up to date.
        }

        _ => {
            panic!("Unknown database version: {}", version);
        }
    }

    //println!("Database path: {}", &db_path.display());

    Ok((conn, db_path))
}

fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        -- COMMAND 1: Core Relational Table
        CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            file_path TEXT NOT NULL UNIQUE,
            file_name TEXT NOT NULL,
            extension TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            mime_type TEXT,
            category TEXT,
            created_at DATETIME,
            modified_at DATETIME,
            indexed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            ai_status TEXT DEFAULT 'pending',
            ai_error TEXT,
            last_accessed_at DATETIME
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

        CREATE TRIGGER IF NOT EXISTS files_ai AFTER INSERT ON files BEGIN
            INSERT INTO files_fts(rowid, id, content_text) VALUES (new.rowid, new.id, NULL);
        END;

        CREATE TRIGGER IF NOT EXISTS files_ad AFTER DELETE ON files BEGIN
            INSERT INTO files_fts(files_fts, rowid, id, content_text) VALUES('delete', old.rowid, old.id, NULL);
        END;

        CREATE TRIGGER IF NOT EXISTS files_au AFTER UPDATE ON files BEGIN
        UPDATE files_fts SET content_text = new.content_text, ai_summary = new.ai_summary, ai_keywords = new.ai_keywords WHERE rowid = new.rowid;
        END;

        -- COMMAND 3: Vector Search Tables (Enabled by sqlite3_auto_extension above)
        CREATE VIRTUAL TABLE IF NOT EXISTS files_vec USING vec0(
            id TEXT PRIMARY KEY,
            embedding float[768]
        );
        "#,
    )?;
    Ok(())
}
