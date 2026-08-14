pub mod filters;
pub mod fts;
pub mod fusion;
pub mod metadata;
pub mod vector;

use std::collections::HashMap;
use std::sync::Mutex;

use rusqlite::types::Value;
use rusqlite::{params_from_iter, Connection};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::services::search::fusion::{fuse, EngineKind, EngineVote, FusedScore, RankedFile};
use crate::services::search::vector::VectorEngine;

pub use filters::SearchFilters;

pub trait SearchEngine {
    fn kind(&self) -> EngineKind;

    /// Each engine locks `conn` itself, only around its SQL. Any external I/O
    /// (e.g. the vector engine's Ollama call) must happen before the lock so the
    /// connection is never held across an await.
    async fn search(
        &self,
        conn: &Mutex<Connection>,
        query: &str,
        filters: &SearchFilters,
    ) -> Result<Vec<RankedFile>, String>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchResult {
    pub file_id: i64,
    pub file_path: String,
    pub file_name: String,
    pub extension: String,
    pub category: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: i64,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub unavailable: Vec<String>,
}

/// Runs the engines sequentially, drops and flags any that error, fuses the
/// survivors with RRF, then hydrates the top `limit` rows.
pub async fn search(
    conn: &Mutex<Connection>,
    query: &str,
    filters: &SearchFilters,
    limit: usize,
    vector: &VectorEngine,
) -> Result<SearchResponse, String> {
    let mut votes = Vec::new();
    let mut unavailable = Vec::new();

    collect_vote(
        crate::services::search::metadata::MetadataEngine
            .search(conn, query, filters)
            .await,
        EngineKind::Metadata,
        &mut votes,
        &mut unavailable,
    );
    collect_vote(
        crate::services::search::fts::FtsEngine
            .search(conn, query, filters)
            .await,
        EngineKind::Fts,
        &mut votes,
        &mut unavailable,
    );
    collect_vote(
        vector.search(conn, query, filters).await,
        EngineKind::Vector,
        &mut votes,
        &mut unavailable,
    );

    let fused = fuse(&votes);
    let top: Vec<FusedScore> = fused.into_iter().take(limit).collect();
    let results = hydrate(conn, &top)?;

    Ok(SearchResponse {
        results,
        unavailable,
    })
}

fn collect_vote(
    result: Result<Vec<RankedFile>, String>,
    kind: EngineKind,
    votes: &mut Vec<EngineVote>,
    unavailable: &mut Vec<String>,
) {
    match result {
        Ok(ranked) if !ranked.is_empty() => votes.push(EngineVote { kind, ranked }),
        Ok(_) => {}
        Err(_) => unavailable.push(kind.to_string()),
    }
}

fn hydrate(conn: &Mutex<Connection>, top: &[FusedScore]) -> Result<Vec<SearchResult>, String> {
    if top.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = top
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "
        SELECT id, file_path, file_name, extension, category, mime_type,
               file_size, created_at, modified_at
        FROM files
        WHERE id IN ({placeholders})
        "
    );
    let params: Vec<Value> = top.iter().map(|f| Value::Integer(f.file_id)).collect();

    let conn = conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params_from_iter(params), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut by_id: HashMap<
        i64,
        (
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            i64,
            Option<String>,
            Option<String>,
        ),
    > = HashMap::new();
    for row in rows {
        let (id, path, name, ext, category, mime, size, created, modified) =
            row.map_err(|e| e.to_string())?;
        by_id.insert(id, (path, name, ext, category, mime, size, created, modified));
    }

    let mut results = Vec::new();
    for score in top {
        if let Some((path, name, ext, category, mime, size, created, modified)) =
            by_id.remove(&score.file_id)
        {
            results.push(SearchResult {
                file_id: score.file_id,
                file_path: path,
                file_name: name,
                extension: ext,
                category,
                mime_type: mime,
                file_size: size,
                created_at: created,
                modified_at: modified,
                score: score.score,
            });
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ai::client::OllamaClient;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn load_vec_extension() {
        unsafe {
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
        }
    }

    fn setup() -> Mutex<Connection> {
        load_vec_extension();
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL,
                file_name TEXT NOT NULL,
                extension TEXT NOT NULL,
                mime_type TEXT,
                category TEXT,
                file_size INTEGER NOT NULL,
                modified_at DATETIME,
                created_at DATETIME
            );
            CREATE VIRTUAL TABLE files_fts USING fts5(
                id UNINDEXED,
                content_text,
                ai_summary,
                ai_keywords,
                content='files',
                content_rowid='rowid'
            );
            CREATE VIRTUAL TABLE files_vec USING vec0(
                id INTEGER PRIMARY KEY,
                embedding float[768]
            );
            ",
        )
        .unwrap();
        Mutex::new(conn)
    }

    fn insert_file(conn: &Connection, name: &str, ext: &str, modified: &str) -> i64 {
        conn.execute(
            "INSERT INTO files (file_path, file_name, extension, file_size, modified_at)
             VALUES (?1, ?2, ?3, 1024, ?4)",
            rusqlite::params![format!("/a/{name}"), name, ext, modified],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn insert_fts(conn: &Connection, id: i64, content: &str) {
        conn.execute(
            "INSERT INTO files_fts(rowid, content_text, ai_summary, ai_keywords)
             VALUES (?1, ?2, NULL, NULL)",
            rusqlite::params![id, content],
        )
        .unwrap();
    }

    fn insert_vec(conn: &Connection, id: i64, dim: usize) {
        let mut v = vec![0.0f32; 768];
        v[dim] = 1.0;
        let json = serde_json::to_string(&v).unwrap();
        conn.execute(
            "INSERT INTO files_vec(id, embedding) VALUES (?1, ?2)",
            rusqlite::params![id, json],
        )
        .unwrap();
    }

    async fn embed_mock(server: &MockServer, dim: usize) {
        let mut v = vec![0.0f32; 768];
        v[dim] = 1.0;
        Mock::given(method("POST"))
            .and(path("/api/embed"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "embeddings": [v]
            })))
            .mount(server)
            .await;
    }

    #[tokio::test]
    async fn metadata_alone_serves_results_when_others_silent() {
        let conn = setup();
        {
            let guard = conn.lock().unwrap();
            insert_file(&guard, "report.txt", "txt", "2024-01-01T00:00:00Z");
            insert_file(&guard, "notes.md", "md", "2024-02-01T00:00:00Z");
        }

        let mock_server = MockServer::start().await;
        embed_mock(&mock_server, 0).await;

        let vector = VectorEngine {
            client: OllamaClient::new(mock_server.uri()),
        };
        let response = search(&conn, "report", &SearchFilters::default(), 10, &vector)
            .await
            .unwrap();

        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].file_name, "report.txt");
        assert!(response.unavailable.is_empty());
    }

    #[tokio::test]
    async fn vector_failure_is_skipped_and_flagged() {
        let conn = setup();
        {
            let guard = conn.lock().unwrap();
            insert_file(&guard, "report.txt", "txt", "2024-01-01T00:00:00Z");
        }

        let vector = VectorEngine {
            client: OllamaClient::new("http://localhost:1"),
        };
        let response = search(&conn, "report", &SearchFilters::default(), 10, &vector)
            .await
            .unwrap();

        assert_eq!(response.results.len(), 1);
        assert_eq!(response.unavailable, vec!["vector"]);
    }

    #[tokio::test]
    async fn consensus_ranks_a_file_above_single_engine_vote() {
        let conn = setup();
        let (fts_id, vec_id) = {
            let guard = conn.lock().unwrap();
            let fts_id = insert_file(&guard, "alpha.txt", "txt", "2024-01-01T00:00:00Z");
            insert_fts(&guard, fts_id, "rust borrow checker");
            let vec_id = insert_file(&guard, "beta.txt", "txt", "2024-01-01T00:00:00Z");
            insert_vec(&guard, vec_id, 0);
            (fts_id, vec_id)
        };

        let mock_server = MockServer::start().await;
        embed_mock(&mock_server, 0).await;

        let vector = VectorEngine {
            client: OllamaClient::new(mock_server.uri()),
        };
        let response = search(&conn, "rust", &SearchFilters::default(), 10, &vector)
            .await
            .unwrap();

        assert_eq!(response.results.len(), 2);
        assert_eq!(response.results[0].file_id, fts_id);
        assert_eq!(response.results[1].file_id, vec_id);
    }

    #[tokio::test]
    async fn filters_narrow_all_engines() {
        let conn = setup();
        let md_id = {
            let guard = conn.lock().unwrap();
            let md_id = insert_file(&guard, "report.md", "md", "2024-01-01T00:00:00Z");
            insert_fts(&guard, md_id, "rust");
            insert_file(&guard, "report.txt", "txt", "2024-01-01T00:00:00Z");
            md_id
        };

        let mock_server = MockServer::start().await;
        embed_mock(&mock_server, 0).await;

        let vector = VectorEngine {
            client: OllamaClient::new(mock_server.uri()),
        };
        let filters = SearchFilters {
            extensions: vec!["md".to_string()],
            ..Default::default()
        };
        let response = search(&conn, "report", &filters, 10, &vector)
            .await
            .unwrap();

        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].file_id, md_id);
    }

    #[tokio::test]
    async fn empty_query_returns_empty_response() {
        let conn = setup();
        let mock_server = MockServer::start().await;
        embed_mock(&mock_server, 0).await;

        let vector = VectorEngine {
            client: OllamaClient::new(mock_server.uri()),
        };
        let response = search(&conn, "   ", &SearchFilters::default(), 10, &vector)
            .await
            .unwrap();

        assert!(response.results.is_empty());
        assert!(response.unavailable.is_empty());
    }
}
