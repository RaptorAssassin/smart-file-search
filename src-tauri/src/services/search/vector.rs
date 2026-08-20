use std::sync::{Arc, Mutex};

use rusqlite::types::Value;
use rusqlite::{params_from_iter, Connection};

use crate::commands::config::models::AiConfig;
use crate::services::ai::client::OllamaClient;
use crate::services::search::filters::SearchFilters;
use crate::services::search::fusion::{EngineKind, RankedFile};
use crate::services::search::SearchEngine;
use crate::services::usage::UsageCounters;

const KNN_CANDIDATES: i64 = 200;

pub struct VectorEngine {
    pub client: OllamaClient,
    pub embeddings_enabled: bool,
}

impl VectorEngine {
    pub fn new() -> Self {
        Self {
            client: OllamaClient::new("http://localhost:11434"),
            embeddings_enabled: true,
        }
    }

    pub fn from_config(ai: &AiConfig, usage: Option<Arc<UsageCounters>>) -> Self {
        let mut client = OllamaClient::with_usage(&ai.ollama_url, usage);
        client.llm_model = ai.ollama_model.clone();
        client.embed_model = ai.embed_model.clone();
        Self {
            client,
            embeddings_enabled: ai.embeddings_enabled,
        }
    }
}

impl SearchEngine for VectorEngine {
    fn kind(&self) -> EngineKind {
        EngineKind::Vector
    }

    async fn search(
        &self,
        conn: &Mutex<Connection>,
        query: &str,
        filters: &SearchFilters,
    ) -> Result<Vec<RankedFile>, String> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }

        if !self.embeddings_enabled {
            return Ok(Vec::new());
        }

        // External I/O happens before the lock so the connection is never held
        // across an await. Ollama down => Err, which the orchestrator turns into
        // a skipped + flagged engine, never a failed search.
        let embedding = self.client.generate_embedding(query).await?;

        let (filter_sql, filter_params) = filters.to_where_sql();
        let id_subquery = if filter_sql.is_empty() {
            String::new()
        } else {
            format!(" AND id IN (SELECT id FROM files WHERE {filter_sql})")
        };

        let sql = format!(
            "
            SELECT id, distance
            FROM files_vec
            WHERE embedding MATCH ?1 AND k = {KNN_CANDIDATES}
            {id_subquery}
            ORDER BY distance ASC
            "
        );

        let query_vec = format!(
            "[{}]",
            embedding
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let mut params: Vec<Value> = vec![Value::Text(query_vec)];
        params.extend(filter_params);

        let conn = conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params_from_iter(params), |row| row.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;

        let mut ranked = Vec::new();
        for row in rows {
            ranked.push(RankedFile {
                file_id: row.map_err(|e| e.to_string())?,
            });
        }
        Ok(ranked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
                modified_at DATETIME,
                created_at DATETIME
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

    fn insert_vec(conn: &Connection, name: &str, ext: &str, embedding: &[f32]) -> i64 {
        conn.execute(
            "INSERT INTO files (file_path, file_name, extension, modified_at)
             VALUES (?1, ?2, ?3, '2024-01-01T00:00:00Z')",
            rusqlite::params![format!("/a/{name}"), name, ext],
        )
        .unwrap();
        let id = conn.last_insert_rowid();
        let json = serde_json::to_string(embedding).unwrap();
        conn.execute(
            "INSERT INTO files_vec(id, embedding) VALUES (?1, ?2)",
            rusqlite::params![id, json],
        )
        .unwrap();
        id
    }

    fn unit_at(dim: usize) -> Vec<f32> {
        let mut v = vec![0.0f32; 768];
        v[dim] = 1.0;
        v
    }

    fn ids(result: &[RankedFile]) -> Vec<i64> {
        result.iter().map(|r| r.file_id).collect()
    }

    #[tokio::test]
    async fn empty_query_returns_no_votes() {
        let conn = setup();
        let engine = VectorEngine::new();
        let result = engine.search(&conn, "  ", &SearchFilters::default()).await;
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn disabled_embeddings_return_successful_empty() {
        let conn = setup();
        let engine = VectorEngine {
            client: OllamaClient::new("http://localhost:1"),
            embeddings_enabled: false,
        };
        let result = engine.search(&conn, "hello", &SearchFilters::default()).await;
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn ollama_down_propagates_error() {
        let conn = setup();
        let engine = VectorEngine {
            client: OllamaClient::new("http://localhost:1"),
            embeddings_enabled: true,
        };
        let result = engine.search(&conn, "hello", &SearchFilters::default()).await;
        assert!(result.is_err(), "expected an error when Ollama is unreachable");
    }

    #[tokio::test]
    async fn empty_vec_table_is_successful_empty() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/embed"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "embeddings": [unit_at(0)]
            })))
            .mount(&mock_server)
            .await;

        let conn = setup();
        let engine = VectorEngine {
            client: OllamaClient::new(mock_server.uri()),
            embeddings_enabled: true,
        };
        let result = engine.search(&conn, "hello", &SearchFilters::default()).await;
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn nearest_embedding_ranks_first() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/embed"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "embeddings": [unit_at(0)]
            })))
            .mount(&mock_server)
            .await;

        let conn = setup();
        insert_vec(&*conn.lock().unwrap(), "a.txt", "txt", &unit_at(0));
        let b_id = insert_vec(&*conn.lock().unwrap(), "b.txt", "txt", &unit_at(1));

        let engine = VectorEngine {
            client: OllamaClient::new(mock_server.uri()),
            embeddings_enabled: true,
        };
        let result = engine.search(&conn, "hello", &SearchFilters::default()).await;
        let ranked = result.unwrap();
        assert_eq!(ids(&ranked)[0], 1);
        assert_eq!(ids(&ranked)[1], b_id);
    }

    #[tokio::test]
    async fn filters_restrict_knn_candidates() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/embed"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "embeddings": [unit_at(0)]
            })))
            .mount(&mock_server)
            .await;

        let conn = setup();
        insert_vec(&*conn.lock().unwrap(), "a.txt", "txt", &unit_at(0));
        insert_vec(&*conn.lock().unwrap(), "b.md", "md", &unit_at(1));

        let engine = VectorEngine {
            client: OllamaClient::new(mock_server.uri()),
            embeddings_enabled: true,
        };
        let filters = SearchFilters {
            extensions: vec!["txt".to_string()],
            ..Default::default()
        };
        let result = engine.search(&conn, "hello", &filters).await;
        assert_eq!(ids(&result.unwrap()), vec![1]);
    }
}