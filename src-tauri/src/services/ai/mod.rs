pub mod client;
pub mod openai;
pub mod pipeline;
pub mod process;
pub mod prompts;

use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::services::database::DbState;
use crate::services::indexer::blacklist::Blacklist;

pub const AI_QUEUE_CAPACITY: usize = 10_000;

/// Spawns the AI worker pool and queues any files left pending from an earlier run.
pub fn start_ai_processing(
    app_handle: AppHandle,
    blacklist: Arc<Blacklist>,
    tx: tokio::sync::mpsc::Sender<i64>,
    rx: tokio::sync::mpsc::Receiver<i64>,
) {
    tauri::async_runtime::spawn(async move {
        let rx = Arc::new(tokio::sync::Mutex::new(rx));

        for _ in 0..worker_count() {
            let rx = Arc::clone(&rx);
            let app_handle = app_handle.clone();
            let blacklist = Arc::clone(&blacklist);
            tauri::async_runtime::spawn(async move {
                loop {
                    let row_id = match rx.lock().await.recv().await {
                        Some(id) => id,
                        None => break,
                    };
                    if let Err(err) =
                        process::ai_process_file(&app_handle, &blacklist, row_id).await
                    {
                        eprintln!("AI processing failed for row {row_id}: {err}");
                        if let Err(e) = process::mark_error(&app_handle, row_id, &err).await {
                            eprintln!("Failed to record error for row {row_id}: {e}");
                        }
                    }
                }
            });
        }

        for row_id in drain_rows_with_status(&app_handle, "pending").await {
            if tx.send(row_id).await.is_err() {
                break;
            }
        }
        for row_id in drain_rows_with_status(&app_handle, "error").await {
            if tx.send(row_id).await.is_err() {
                break;
            }
        }
    });
}

/// Picks how many AI workers to run, capped so weak machines aren't swamped.
fn worker_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2)
        .min(4)
}

/// Collects the ids of files that still need AI processing at startup.
async fn drain_rows_with_status(app_handle: &AppHandle, status: &str) -> Vec<i64> {
    let state = match app_handle.try_state::<DbState>() {
        Some(state) => state,
        None => return Vec::new(),
    };
    let conn = match state.conn.lock() {
        Ok(conn) => conn,
        Err(_) => return Vec::new(),
    };
    let mut stmt = match conn.prepare("SELECT id FROM files WHERE ai_status = ?1 ORDER BY id") {
        Ok(stmt) => stmt,
        Err(_) => return Vec::new(),
    };
    let mut rows = match stmt.query([status]) {
        Ok(rows) => rows,
        Err(_) => return Vec::new(),
    };
    let mut ids = Vec::new();
    while let Ok(Some(row)) = rows.next() {
        if let Ok(id) = row.get::<_, i64>(0) {
            ids.push(id);
        }
    }
    ids
}
