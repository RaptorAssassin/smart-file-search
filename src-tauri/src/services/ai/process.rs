use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

use rusqlite::params;

use crate::commands::config::models::AiProvider;
use crate::services::ai::client::OllamaClient;
use crate::services::ai::openai::OpenAiClient;
use crate::services::ai::pipeline::{self, PipelineKind};
use crate::services::database::DbState;
use crate::services::indexer::blacklist::Blacklist;
use crate::services::usage::UsageCounters;
use crate::AppState;

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

    // Persist the extracted text before any AI work so content search keeps
    // working even when the model calls below fail (Ollama down, bad replies).
    if !content_text.is_empty() {
        persist_content(app_handle, row_id, &content_text).await?;
    }

    let ai_config = app_handle
        .try_state::<AppState>()
        .map(|state| state.config_manager.config.read().unwrap().clone().settings.ai)
        .unwrap_or_default();

    let usage = app_handle
        .try_state::<Arc<UsageCounters>>()
        .map(|state| Arc::clone(&state));

    // Keyword/summary/embedding generation is best-effort: a failure never
    // discards the content already persisted above. Failures are recorded so
    // the row is re-queued on the next startup to fill in the AI fields.
    let mut keywords = Vec::new();
    let mut summary = String::new();
    let mut ai_errors: Vec<String> = Vec::new();

    if let Some(text) = &model_text {
        let result: Result<(Vec<String>, String), String> = match ai_config.provider {
            AiProvider::Ollama => {
                let mut client = OllamaClient::with_usage(&ai_config.ollama_url, usage.clone());
                client.llm_model = ai_config.ollama_model.clone();
                client.embed_model = ai_config.embed_model.clone();
                async {
                    let keywords = client.generate_keywords(text).await?;
                    let summary = client.generate_summary(text).await?;
                    Ok::<_, String>((keywords, summary))
                }
                .await
            }
            AiProvider::Custom => {
                let client = OpenAiClient::with_usage(
                    &ai_config.custom_endpoint,
                    &ai_config.custom_api_key,
                    &ai_config.custom_model,
                    usage.clone(),
                );
                async {
                    let keywords = client.generate_keywords(text).await?;
                    let summary = client.generate_summary(text).await?;
                    Ok::<_, String>((keywords, summary))
                }
                .await
            }
        };
        match result {
            Ok((kw, sum)) => {
                keywords = kw;
                summary = sum;
            }
            Err(err) => ai_errors.push(err),
        }
    }

    let mut embedding: Vec<f32> = Vec::new();
    if ai_config.embeddings_enabled {
        let mut client = OllamaClient::with_usage(&ai_config.ollama_url, usage);
        client.llm_model = ai_config.ollama_model.clone();
        client.embed_model = ai_config.embed_model.clone();

        let attempt: Result<Vec<f32>, String> = if let Some(text) = &model_text {
            client.generate_embedding(text).await
        } else if let Some(image) = &image_path {
            match image_to_base64(image) {
                Ok(b64) => client.generate_embedding_from_image(&b64).await,
                Err(err) => Err(err),
            }
        } else {
            Ok(Vec::new())
        };

        match attempt {
            Ok(vec) => embedding = vec,
            Err(err) => ai_errors.push(err),
        }
    }

    update_row(
        app_handle,
        row_id,
        &content_text,
        &summary,
        &keywords,
        &embedding,
    )
    .await?;

    if let Some(err) = ai_errors.first() {
        mark_error(app_handle, row_id, err).await?;
    }

    Ok(())
}

/// Writes the extracted text into the row so FTS search can find it even if
/// the AI enrichment steps fail later.
async fn persist_content(
    app_handle: &AppHandle,
    row_id: i64,
    content_text: &str,
) -> Result<(), String> {
    let state = app_handle
        .try_state::<DbState>()
        .ok_or_else(|| "Database state not available".to_string())?;
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE files SET content_text = ?1 WHERE id = ?2",
        params![content_text, row_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
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

    let mut produced_ai_data = false;

    if content_text.is_empty() && summary.is_empty() && joined_keywords.is_empty() {
        conn.execute(
            "UPDATE files SET ai_status = 'done' WHERE id = ?1",
            params![row_id],
        )
        .map_err(|e| e.to_string())?;
    } else {
        produced_ai_data = true;
        conn.execute(
            "UPDATE files SET content_text = ?1, ai_summary = ?2, ai_keywords = ?3, ai_status = 'done' WHERE id = ?4",
            params![content_text, summary, joined_keywords, row_id],
        )
        .map_err(|e| e.to_string())?;
    }

    if !embedding.is_empty() {
        produced_ai_data = true;
        let embedding_json = serde_json::to_string(embedding).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO files_vec(id, embedding) VALUES (?1, ?2)",
            params![row_id, embedding_json],
        )
        .map_err(|e| e.to_string())?;
    }

    drop(conn);

    if produced_ai_data {
        if let Some(usage) = app_handle.try_state::<Arc<UsageCounters>>() {
            usage.incr_files_ai_indexed();
        }
    }

    Ok(())
}
