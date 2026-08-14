use std::sync::Mutex;

use rusqlite::types::Value;
use rusqlite::{params_from_iter, Connection};

use crate::services::search::filters::SearchFilters;
use crate::services::search::fusion::{EngineKind, RankedFile};
use crate::services::search::SearchEngine;

const MAX_MATCHES: i64 = 200;

/// The tier CASE expression uses one `?` per branch; keep this in sync.
const MATCH_PARAM_COUNT: usize = 4;

pub struct MetadataEngine;

impl SearchEngine for MetadataEngine {
    fn kind(&self) -> EngineKind {
        EngineKind::Metadata
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

        let (filter_sql, filter_params) = filters.to_where_sql();
        let where_clause = if filter_sql.is_empty() {
            String::new()
        } else {
            format!(" WHERE {filter_sql}")
        };

        let sql = format!(
            "
            SELECT id FROM (
                SELECT id, modified_at,
                    CASE
                        WHEN lower(file_name) = lower(?) THEN 0
                        WHEN lower(file_name) LIKE lower(?) || '%' THEN 1
                        WHEN instr(lower(file_name), lower(?)) > 0 THEN 2
                        WHEN instr(lower(
                            IFNULL(file_path, '') || IFNULL(extension, '') ||
                            IFNULL(mime_type, '') || IFNULL(category, '')
                        ), lower(?)) > 0 THEN 3
                        ELSE 4
                    END AS tier
                FROM files
                {where_clause}
            ) WHERE tier < 4
            ORDER BY tier ASC, modified_at DESC, id ASC
            LIMIT {MAX_MATCHES}
            "
        );

        let mut params: Vec<Value> = vec![Value::Text(query.to_string()); MATCH_PARAM_COUNT];
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

    fn setup() -> Mutex<Connection> {
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
            ",
        )
        .unwrap();
        Mutex::new(conn)
    }

    fn insert(
        conn: &Connection,
        name: &str,
        path: &str,
        ext: &str,
        mime: Option<&str>,
        modified: &str,
    ) {
        conn.execute(
            "INSERT INTO files (file_path, file_name, extension, mime_type, modified_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![path, name, ext, mime, modified],
        )
        .unwrap();
    }

    async fn run(
        conn: &Mutex<Connection>,
        query: &str,
        filters: SearchFilters,
    ) -> Vec<RankedFile> {
        MetadataEngine
            .search(conn, query, &filters)
            .await
            .unwrap()
    }

    fn ids(result: &[RankedFile]) -> Vec<i64> {
        result.iter().map(|r| r.file_id).collect()
    }

    #[tokio::test]
    async fn empty_query_returns_no_votes() {
        let conn = setup();
        insert(&*conn.lock().unwrap(), "report.txt", "/a/report.txt", "txt", None, "2024-01-01T00:00:00Z");
        let result = run(&conn, "   ", SearchFilters::default()).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn exact_match_ranks_first() {
        let conn = setup();
        insert(&*conn.lock().unwrap(), "report.txt", "/a/report.txt", "txt", None, "2024-01-01T00:00:00Z");
        insert(&*conn.lock().unwrap(), "myreport.txt", "/a/myreport.txt", "txt", None, "2024-02-01T00:00:00Z");
        let result = run(&conn, "report.txt", SearchFilters::default()).await;
        assert_eq!(ids(&result), vec![1, 2]);
    }

    #[tokio::test]
    async fn prefix_outranks_substring() {
        let conn = setup();
        insert(&*conn.lock().unwrap(), "report-notes.md", "/a/report-notes.md", "md", None, "2024-01-01T00:00:00Z");
        insert(&*conn.lock().unwrap(), "annual-report.pdf", "/a/annual-report.pdf", "pdf", None, "2024-02-01T00:00:00Z");
        let result = run(&conn, "report", SearchFilters::default()).await;
        assert_eq!(ids(&result), vec![1, 2]);
    }

    #[tokio::test]
    async fn filename_beats_path_only_match() {
        let conn = setup();
        insert(&*conn.lock().unwrap(), "notes.txt", "/docs/report/notes.txt", "txt", None, "2024-01-01T00:00:00Z");
        insert(&*conn.lock().unwrap(), "report.txt", "/a/report.txt", "txt", None, "2024-02-01T00:00:00Z");
        let result = run(&conn, "report", SearchFilters::default()).await;
        assert_eq!(ids(&result), vec![2, 1]);
    }

    #[tokio::test]
    async fn mime_type_match_is_lowest_tier() {
        let conn = setup();
        insert(&*conn.lock().unwrap(), "README", "/a/README", "", Some("text/report+xml"), "2024-01-01T00:00:00Z");
        insert(&*conn.lock().unwrap(), "report.txt", "/a/report.txt", "txt", None, "2024-02-01T00:00:00Z");
        let result = run(&conn, "report", SearchFilters::default()).await;
        assert_eq!(ids(&result), vec![2, 1]);
    }

    #[tokio::test]
    async fn null_mime_does_not_block_path_match() {
        let conn = setup();
        insert(&*conn.lock().unwrap(), "notes.txt", "/docs/report/notes.txt", "txt", None, "2024-01-01T00:00:00Z");
        let result = run(&conn, "report", SearchFilters::default()).await;
        assert_eq!(ids(&result), vec![1]);
    }

    #[tokio::test]
    async fn filters_are_applied_before_ranking() {
        let conn = setup();
        insert(&*conn.lock().unwrap(), "report.txt", "/a/report.txt", "txt", None, "2024-01-01T00:00:00Z");
        insert(&*conn.lock().unwrap(), "report.md", "/a/report.md", "md", None, "2024-02-01T00:00:00Z");
        let filters = SearchFilters {
            extensions: vec!["md".to_string()],
            ..Default::default()
        };
        let result = run(&conn, "report", filters).await;
        assert_eq!(ids(&result), vec![2]);
    }

    #[tokio::test]
    async fn no_matches_returns_empty() {
        let conn = setup();
        insert(&*conn.lock().unwrap(), "report.txt", "/a/report.txt", "txt", None, "2024-01-01T00:00:00Z");
        let result = run(&conn, "zzzz", SearchFilters::default()).await;
        assert!(result.is_empty());
    }
}