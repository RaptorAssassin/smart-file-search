use rusqlite::types::Value;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Type)]
pub struct SearchFilters {
    pub extensions: Vec<String>,
    pub min_size: Option<i64>,
    pub max_size: Option<i64>,
    pub modified_after: Option<String>,
    pub modified_before: Option<String>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
}

impl SearchFilters {
    pub fn to_where_sql(&self) -> (String, Vec<Value>) {
        let mut predicates: Vec<String> = Vec::new();
        let mut params: Vec<Value> = Vec::new();

        if !self.extensions.is_empty() {
            let placeholders = self
                .extensions
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", ");
            predicates.push(format!("extension IN ({placeholders})"));
            params.extend(
                self.extensions
                    .iter()
                    .cloned()
                    .map(Value::Text),
            );
        }

        if let Some(min) = self.min_size {
            predicates.push("file_size >= ?".to_string());
            params.push(Value::Integer(min));
        }

        if let Some(max) = self.max_size {
            predicates.push("file_size <= ?".to_string());
            params.push(Value::Integer(max));
        }

        if let Some(date) = &self.modified_after {
            predicates.push("modified_at >= ?".to_string());
            params.push(Value::Text(date.clone()));
        }

        if let Some(date) = &self.modified_before {
            predicates.push("modified_at <= ?".to_string());
            params.push(Value::Text(date.clone()));
        }

        if let Some(date) = &self.created_after {
            predicates.push("created_at >= ?".to_string());
            params.push(Value::Text(date.clone()));
        }

        if let Some(date) = &self.created_before {
            predicates.push("created_at <= ?".to_string());
            params.push(Value::Text(date.clone()));
        }

        if predicates.is_empty() {
            (String::new(), params)
        } else {
            (predicates.join(" AND "), params)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{params_from_iter, Connection};

    #[test]
    fn no_filters_produce_empty_fragment() {
        let (sql, params) = SearchFilters::default().to_where_sql();
        assert_eq!(sql, "");
        assert!(params.is_empty());
    }

    #[test]
    fn min_size_produces_single_predicate() {
        let filters = SearchFilters {
            min_size: Some(1024),
            ..Default::default()
        };
        let (sql, params) = filters.to_where_sql();
        assert_eq!(sql, "file_size >= ?");
        assert_eq!(params, vec![Value::Integer(1024)]);
    }

    #[test]
    fn extensions_generate_matching_placeholders() {
        let filters = SearchFilters {
            extensions: vec!["pdf".to_string(), "md".to_string(), "txt".to_string()],
            ..Default::default()
        };
        let (sql, params) = filters.to_where_sql();
        assert_eq!(sql, "extension IN (?, ?, ?)");
        assert_eq!(
            params,
            vec![
                Value::Text("pdf".to_string()),
                Value::Text("md".to_string()),
                Value::Text("txt".to_string()),
            ]
        );
    }

    #[test]
    fn empty_extensions_are_skipped() {
        let filters = SearchFilters {
            extensions: vec![],
            min_size: Some(5),
            ..Default::default()
        };
        let (sql, params) = filters.to_where_sql();
        assert_eq!(sql, "file_size >= ?");
        assert_eq!(params, vec![Value::Integer(5)]);
    }

    #[test]
    fn full_combination_joins_with_and_in_order() {
        let filters = SearchFilters {
            extensions: vec!["pdf".to_string()],
            min_size: Some(100),
            max_size: Some(500),
            modified_after: Some("2024-01-01T00:00:00Z".to_string()),
            modified_before: Some("2024-12-31T00:00:00Z".to_string()),
            created_after: Some("2023-01-01T00:00:00Z".to_string()),
            created_before: Some("2023-12-31T00:00:00Z".to_string()),
        };
        let (sql, params) = filters.to_where_sql();
        assert_eq!(
            sql,
            "extension IN (?) AND file_size >= ? AND file_size <= ? AND modified_at >= ? AND modified_at <= ? AND created_at >= ? AND created_at <= ?"
        );
        assert_eq!(params.len(), 7);
    }

    #[test]
    fn fragment_runs_against_real_db() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE files (file_size INTEGER, extension TEXT, modified_at TEXT, created_at TEXT);",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO files VALUES (100, 'pdf', '2024-01-01T00:00:00Z', '2023-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO files VALUES (200, 'md', '2024-02-01T00:00:00Z', '2023-02-01T00:00:00Z')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO files VALUES (300, 'pdf', '2024-03-01T00:00:00Z', '2023-03-01T00:00:00Z')",
            [],
        )
        .unwrap();

        let filters = SearchFilters {
            extensions: vec!["pdf".to_string()],
            min_size: Some(150),
            modified_after: Some("2024-02-15T00:00:00Z".to_string()),
            ..Default::default()
        };
        let (fragment, params) = filters.to_where_sql();
        let sql = if fragment.is_empty() {
            "SELECT extension FROM files".to_string()
        } else {
            format!("SELECT extension FROM files WHERE {fragment}")
        };

        let mut stmt = conn.prepare(&sql).unwrap();
        let rows: Vec<String> = stmt
            .query_map(params_from_iter(params), |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(rows, vec!["pdf"]);
    }
}