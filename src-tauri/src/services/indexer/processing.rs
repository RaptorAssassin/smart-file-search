use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension, Result};
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

pub enum ProcessOutcome {
    NeedsAi(i64),
    Unchanged,
}

pub async fn process_file(
    app_handle: &AppHandle,
    path: &PathBuf,
    scan_id: i64,
) -> Result<ProcessOutcome, String> {
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => return Err(format!("Failed to read metadata for {:?}: {}", path, e)),
    };

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_string();

    let file_path = path.to_string_lossy().to_string();

    let file_size = metadata.len() as i64;

    #[cfg(unix)]
    let inode = Some(metadata.ino() as i64);
    #[cfg(not(unix))]
    let inode: Option<i64> = None;

    let created_at = metadata
        .created()
        .ok()
        .map(|t| DateTime::<Utc>::from(t).to_rfc3339());

    let modified_at = metadata
        .modified()
        .ok()
        .map(|t| DateTime::<Utc>::from(t).to_rfc3339());

    let mime_type = mime_guess::from_path(path).first().map(|m| m.to_string());

    let file_hash = calculate_file_hash(path).unwrap_or_default();

    let indexed_at = Some(Utc::now().to_rfc3339());

    let db_state = app_handle
        .try_state::<crate::services::database::DbState>()
        .ok_or_else(|| "Database state not available".to_string())?;
    let conn = db_state.conn.lock().map_err(|e| e.to_string())?;

    let existing = conn
        .query_row(
            "SELECT id, file_hash FROM files WHERE file_path = ?1",
            [&file_path],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    match existing {
        None => {
            conn.execute(
                "INSERT INTO files (file_path, file_name, file_hash, extension, file_size, inode, mime_type, created_at, modified_at, indexed_at, last_seen_scan_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    file_path,
                    file_name,
                    file_hash,
                    extension,
                    file_size,
                    inode,
                    mime_type,
                    created_at,
                    modified_at,
                    indexed_at,
                    scan_id
                ],
            )
            .map_err(|e| e.to_string())?;

            Ok(ProcessOutcome::NeedsAi(conn.last_insert_rowid()))
        }
        Some((row_id, existing_hash)) if existing_hash == file_hash => {
            conn.execute(
                "UPDATE files SET last_seen_scan_id = ?1, modified_at = ?2, file_size = ?3
                 WHERE id = ?4",
                params![scan_id, modified_at, file_size, row_id],
            )
            .map_err(|e| e.to_string())?;

            Ok(ProcessOutcome::Unchanged)
        }
        Some((row_id, _)) => {
            conn.execute(
                "UPDATE files SET file_hash = ?1, file_size = ?2, modified_at = ?3,
                    mime_type = ?4, indexed_at = ?5, last_seen_scan_id = ?6,
                    ai_status = 'pending', ai_error = NULL, content_text = NULL,
                    ai_summary = NULL, ai_keywords = NULL
                 WHERE id = ?7",
                params![file_hash, file_size, modified_at, mime_type, indexed_at, scan_id, row_id],
            )
            .map_err(|e| e.to_string())?;

            conn.execute("DELETE FROM files_vec WHERE id = ?1", params![row_id])
                .map_err(|e| e.to_string())?;

            Ok(ProcessOutcome::NeedsAi(row_id))
        }
    }
}

pub fn next_scan_id(app_handle: &AppHandle) -> i64 {
    let Some(state) = app_handle.try_state::<crate::services::database::DbState>() else {
        return 1;
    };
    let Ok(conn) = state.conn.lock() else {
        return 1;
    };
    conn.query_row(
        "SELECT COALESCE(MAX(last_seen_scan_id), 0) + 1 FROM files",
        [],
        |row| row.get(0),
    )
    .unwrap_or(1)
}

pub fn cleanup_missing_files(app_handle: &AppHandle, scan_id: i64) -> Result<(), String> {
    let Some(state) = app_handle.try_state::<crate::services::database::DbState>() else {
        return Ok(());
    };
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM files WHERE last_seen_scan_id < ?1",
        params![scan_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn calculate_file_hash(path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 65536];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}
