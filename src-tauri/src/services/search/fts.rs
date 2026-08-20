use std::sync::Mutex;

use rusqlite::types::Value;
use rusqlite::{params_from_iter, Connection};

use crate::services::search::filters::SearchFilters;
use crate::services::search::fusion::{EngineKind, RankedFile};
use crate::services::search::SearchEngine;

const MAX_MATCHES: i64 = 200;

pub struct FtsEngine;

impl SearchEngine for FtsEngine {
    fn kind(&self) -> EngineKind {
        EngineKind::Fts
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

        let terms: Vec<&str> = query.split_whitespace().collect();
        let match_query = terms
            .iter()
            .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" OR ");
        let (filter_sql, filter_params) = filters.to_where_sql();
        let filter_clause = if filter_sql.is_empty() {
            String::new()
        } else {
            format!(" AND {filter_sql}")
        };

        let sql = format!(
            "
            SELECT f.id
            FROM files_fts ft
            JOIN files f ON f.id = ft.rowid
            WHERE files_fts MATCH ?1{filter_clause}
            ORDER BY ft.rank
            LIMIT {MAX_MATCHES}
            "
        );

        let mut params: Vec<Value> = vec![Value::Text(match_query)];
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
            CREATE VIRTUAL TABLE files_fts USING fts5(
                id UNINDEXED,
                content_text,
                ai_summary,
                ai_keywords,
                content='files',
                content_rowid='rowid'
            );
            ",
        )
        .unwrap();
        Mutex::new(conn)
    }

    fn insert(
        conn: &Connection,
        name: &str,
        ext: &str,
        content: Option<&str>,
        summary: Option<&str>,
        keywords: Option<&str>,
    ) -> i64 {
        conn.execute(
            "INSERT INTO files (file_path, file_name, extension, modified_at)
             VALUES (?1, ?2, ?3, '2024-01-01T00:00:00Z')",
            rusqlite::params![format!("/a/{name}"), name, ext],
        )
        .unwrap();
        let id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO files_fts(rowid, content_text, ai_summary, ai_keywords)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id, content, summary, keywords],
        )
        .unwrap();
        id
    }

    async fn run(conn: &Mutex<Connection>, query: &str, filters: SearchFilters) -> Vec<RankedFile> {
        FtsEngine.search(conn, query, &filters).await.unwrap()
    }

    fn ids(result: &[RankedFile]) -> Vec<i64> {
        result.iter().map(|r| r.file_id).collect()
    }

    #[tokio::test]
    async fn empty_query_returns_no_votes() {
        let conn = setup();
        insert(
            &*conn.lock().unwrap(),
            "a.txt",
            "txt",
            Some("rust"),
            None,
            None,
        );
        let result = run(&conn, "  ", SearchFilters::default()).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn matches_content_text() {
        let conn = setup();
        let id = insert(
            &*conn.lock().unwrap(),
            "a.txt",
            "txt",
            Some("rust borrow checker"),
            None,
            None,
        );
        let result = run(&conn, "borrow", SearchFilters::default()).await;
        assert_eq!(ids(&result), vec![id]);
    }

    #[tokio::test]
    async fn multi_term_query_matches_any_term_and_ranks_both_terms_first() {
        let conn = setup();
        let both_id = insert(
            &*conn.lock().unwrap(),
            "both.txt",
            "txt",
            Some("rust rabbit borrow"),
            None,
            None,
        );
        let rabbit_id = insert(
            &*conn.lock().unwrap(),
            "rabbit.txt",
            "txt",
            Some("rabbit hops"),
            None,
            None,
        );
        let fox_id = insert(
            &*conn.lock().unwrap(),
            "fox.txt",
            "txt",
            Some("fox jumps"),
            None,
            None,
        );
        let result = run(&conn, "rust rabbit", SearchFilters::default()).await;
        let ids = ids(&result);
        assert!(ids.contains(&both_id));
        assert!(ids.contains(&rabbit_id));
        assert!(!ids.contains(&fox_id));
        assert_eq!(ids[0], both_id);
    }

    #[tokio::test]
    async fn update_to_files_syncs_content_into_fts() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL,
                file_name TEXT NOT NULL,
                extension TEXT NOT NULL,
                content_text TEXT,
                ai_summary TEXT,
                ai_keywords TEXT,
                modified_at DATETIME
            );
            CREATE VIRTUAL TABLE files_fts USING fts5(
                id UNINDEXED,
                content_text,
                ai_summary,
                ai_keywords,
                content='files',
                content_rowid='rowid'
            );
            CREATE TRIGGER files_ai AFTER INSERT ON files BEGIN
                INSERT INTO files_fts(rowid, content_text, ai_summary, ai_keywords)
                VALUES (new.id, new.content_text, new.ai_summary, new.ai_keywords);
            END;
            CREATE TRIGGER files_au AFTER UPDATE ON files BEGIN
                INSERT INTO files_fts(files_fts, rowid, content_text, ai_summary, ai_keywords)
                VALUES('delete', old.id, old.content_text, old.ai_summary, old.ai_keywords);
                INSERT INTO files_fts(rowid, content_text, ai_summary, ai_keywords)
                VALUES (new.id, new.content_text, new.ai_summary, new.ai_keywords);
            END;
            ",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO files (file_path, file_name, extension, modified_at)
             VALUES ('/a/test.txt', 'test.txt', 'txt', '2024-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let id = conn.last_insert_rowid();
        conn.execute(
            "UPDATE files SET content_text = 'the quick fox' WHERE id = ?1",
            [id],
        )
        .unwrap();

        let result = FtsEngine
            .search(&Mutex::new(conn), "fox", &SearchFilters::default())
            .await
            .unwrap();
        assert_eq!(ids(&result), vec![id]);
    }

    #[tokio::test]
    async fn no_content_rows_give_successful_empty() {
        let conn = setup();
        insert(
            &*conn.lock().unwrap(),
            "a.txt",
            "txt",
            Some("rust"),
            None,
            None,
        );
        let result = run(&conn, "photosynthesis", SearchFilters::default()).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn empty_fts_table_is_not_an_error() {
        let conn = setup();
        let result = run(&conn, "rust", SearchFilters::default()).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn operator_chars_are_treated_literally() {
        let conn = setup();
        let id = insert(
            &*conn.lock().unwrap(),
            "a.txt",
            "txt",
            Some("C++ pointers"),
            None,
            None,
        );
        let result = run(&conn, "C++", SearchFilters::default()).await;
        assert_eq!(ids(&result), vec![id]);
    }

    #[tokio::test]
    async fn filters_are_applied_via_join() {
        let conn = setup();
        let md_id = insert(
            &*conn.lock().unwrap(),
            "a.md",
            "md",
            Some("rust"),
            None,
            None,
        );
        insert(
            &*conn.lock().unwrap(),
            "a.txt",
            "txt",
            Some("rust"),
            None,
            None,
        );
        let filters = SearchFilters {
            extensions: vec!["md".to_string()],
            ..Default::default()
        };
        let result = run(&conn, "rust", filters).await;
        assert_eq!(ids(&result), vec![md_id]);
    }
}
