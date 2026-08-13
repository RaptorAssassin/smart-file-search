use crate::services::database::DbContent;
use chrono::{DateTime, Utc};
use rusqlite::{params, Result};
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

/// Indexes one file's metadata and returns the row id it was stored under.
pub async fn process_file(app_handle: &AppHandle, path: &PathBuf) -> Result<i64, String> {
    println!("Processing file: {:?}", path);

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
    let inode = None;

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

    let file_data = DbContent {
        file_path,
        file_name,
        file_hash,
        extension,
        file_size,

        inode,
        mime_type,
        category: None,
        content_text: None,
        ai_summary: None,
        ai_keywords: None,
        created_at,
        modified_at,
        indexed_at: Some(Utc::now().to_rfc3339()),
        ai_status: Some("pending".to_string()),
        ai_error: None,
        last_accessed_at: None,
        last_seen_scan_id: None,
    };

    let db_state = app_handle
        .try_state::<crate::services::database::DbState>()
        .ok_or_else(|| "Database state not available".to_string())?;
    let conn = db_state.conn.lock().map_err(|e| e.to_string())?;

    conn.execute("
        INSERT INTO files (file_path, file_name, file_hash, extension, file_size, inode, mime_type, created_at, modified_at, indexed_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)", params![file_data.file_path, file_data.file_name, file_data.file_hash, file_data.extension, file_data.file_size, file_data.inode, file_data.mime_type, file_data.created_at, file_data.modified_at, file_data.indexed_at]).map_err(|e| e.to_string())?;

    let row_id = conn.last_insert_rowid();

    println!("Inserted file data into database: {:#?}", file_data);
    Ok(row_id)
}

/// Hashes a file's full contents with blake3 to use as a change detector.
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
