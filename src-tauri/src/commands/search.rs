use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::services::database::DbState;
use crate::services::search::vector::VectorEngine;
use crate::services::search::{self, SearchFilters, SearchResponse};
use crate::services::usage::UsageCounters;
use crate::AppState;

#[tauri::command]
#[specta::specta]
pub async fn search_files(
    app: State<'_, AppState>,
    usage: State<'_, Arc<UsageCounters>>,
    db: State<'_, DbState>,
    query: String,
    filters: Option<SearchFilters>,
    limit: Option<usize>,
) -> Result<SearchResponse, String> {
    let filters = filters.unwrap_or_default();
    let limit = limit.unwrap_or(50);
    let ai = app.config_manager.config.read().unwrap().clone().settings.ai;
    let vector = VectorEngine::from_config(&ai, Some(Arc::clone(&usage)));
    search::search(&db.conn, &query, &filters, limit, &vector).await
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchFilterOptions {
    pub extensions: Vec<String>,
    pub categories: Vec<String>,
    pub min_size: i64,
    pub max_size: i64,
    pub min_modified_at: Option<String>,
    pub max_modified_at: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub fn search_filter_options(db: State<'_, DbState>) -> Result<SearchFilterOptions, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let extensions = {
        let mut stmt = conn
            .prepare("SELECT DISTINCT extension FROM files WHERE extension != '' ORDER BY extension")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        out
    };

    let categories = {
        let mut stmt = conn
            .prepare("SELECT DISTINCT category FROM files WHERE category IS NOT NULL ORDER BY category")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| row.get::<_, Option<String>>(0))
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            if let Some(category) = row.map_err(|e| e.to_string())? {
                out.push(category);
            }
        }
        out
    };

    let (min_size, max_size) = conn
        .query_row(
            "SELECT MIN(file_size), MAX(file_size) FROM files",
            [],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .map_err(|e| e.to_string())?;

    let (min_modified_at, max_modified_at) = conn
        .query_row(
            "SELECT MIN(modified_at), MAX(modified_at) FROM files",
            [],
            |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .map_err(|e| e.to_string())?;

    Ok(SearchFilterOptions {
        extensions,
        categories,
        min_size,
        max_size,
        min_modified_at,
        max_modified_at,
    })
}
