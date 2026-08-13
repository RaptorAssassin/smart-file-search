use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

use rusqlite::params;

use crate::services::ai::client::OllamaClient;
use crate::services::ai::pipeline::{self, PipelineKind};
use crate::services::database::DbState;
use crate::services::indexer::blacklist::Blacklist;

/// Runs one file through the AI pipeline and writes the results back to the database.
pub async fn ai_process_file(
    app_handle: &AppHandle,
    blacklist: &Blacklist,
    row_id: i64,
) -> Result<(), String> {
    let state = app_handle
        .try_state::<DbState>()
        .ok_or_else(|| "Database state not available".to_string())?;

    let (file_path, extension) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT file_path, extension FROM files WHERE id = ?1",
            params![row_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|e| e.to_string())?
    };

    let path = PathBuf::from(&file_path);

    if blacklist.should_skip_path(&path) {
        return update_row(app_handle, row_id, "", "", &[], &[]).await;
    }

    let kind = pipeline::classify(&extension);

    let mut content_text = String::new();
    let mut model_text = None;
    let mut image_path = None;

    match kind {
        PipelineKind::Text | PipelineKind::Pdf => {
            let extracted = pipeline::extract_text(&path, kind)?;
            if extracted.trim().is_empty() && kind == PipelineKind::Pdf {
                image_path = Some(path);
            } else if !extracted.trim().is_empty() {
                content_text = extracted.clone();
                model_text = Some(pipeline::truncate_head_tail(&extracted));
            }
        }
        PipelineKind::Image => {
            image_path = Some(path);
        }
        PipelineKind::Audio | PipelineKind::Video | PipelineKind::Unsupported => {}
    }

    let client = OllamaClient::new("http://localhost:11434");

    let mut keywords = Vec::new();
    let mut summary = String::new();
    if let Some(text) = &model_text {
        keywords = client.generate_keywords(text).await?;
        summary = client.generate_summary(text).await?;
    }

    let embedding = if let Some(text) = &model_text {
        client.generate_embedding(text).await?
    } else if let Some(image) = &image_path {
        client
            .generate_embedding_from_image(&image_to_base64(image)?)
            .await?
    } else {
        Vec::new()
    };

    update_row(
        app_handle,
        row_id,
        &content_text,
        &summary,
        &keywords,
        &embedding,
    )
    .await
}

/// Marks a file as failed so it isn't picked up again.
pub async fn mark_error(app_handle: &AppHandle, row_id: i64, error: &str) -> Result<(), String> {
    let state = app_handle
        .try_state::<DbState>()
        .ok_or_else(|| "Database state not available".to_string())?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE files SET ai_status = 'error', ai_error = ?1 WHERE id = ?2",
        params![error, row_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Reads an image file and base64-encodes it for the vision models.
fn image_to_base64(path: &Path) -> Result<String, String> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    Ok(STANDARD.encode(bytes))
}

async fn update_row(
    app_handle: &AppHandle,
    row_id: i64,
    content_text: &str,
    summary: &str,
    keywords: &[String],
    embedding: &[f32],
) -> Result<(), String> {
    let state = app_handle
        .try_state::<DbState>()
        .ok_or_else(|| "Database state not available".to_string())?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;

    let joined_keywords = keywords.join(", ");

    if content_text.is_empty() && summary.is_empty() && joined_keywords.is_empty() {
        conn.execute(
            "UPDATE files SET ai_status = 'done' WHERE id = ?1",
            params![row_id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        conn.execute(
            "UPDATE files SET content_text = ?1, ai_summary = ?2, ai_keywords = ?3, ai_status = 'done' WHERE id = ?4",
            params![content_text, summary, joined_keywords, row_id],
        )
        .map_err(|e| e.to_string())?;
    }

    if !embedding.is_empty() {
        let embedding_json = serde_json::to_string(embedding).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO files_vec(rowid, embedding) VALUES (?1, ?2)",
            params![row_id, embedding_json],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
