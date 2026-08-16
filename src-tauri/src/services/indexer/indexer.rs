use crate::services::ai::{self, AI_QUEUE_CAPACITY};
use crate::services::indexer::blacklist::Blacklist;
use crate::services::usage::UsageCounters;
use ignore::{WalkBuilder, WalkState};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::mpsc;

use crate::services::indexer::processing::process_file;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

/// Walks the filesystem and pushes every non-blacklisted file into the indexing queue.
pub fn traverse_system(root: PathBuf, blacklist: Arc<Blacklist>, tx: mpsc::Sender<PathBuf>) {
    // Arc is used to share the blacklist across threads without needing to clone it for each entry
    let filter_blacklist = Arc::clone(&blacklist);

    let walker = WalkBuilder::new(root)
        .threads(std::thread::available_parallelism().unwrap().get())
        .hidden(false) // Include hidden files
        .git_ignore(false)
        .same_file_system(true)
        // Skip blacklisted directories
        .filter_entry(move |entry| !filter_blacklist.should_skip_path(entry.path()))
        .build_parallel();

    let blacklist = Arc::clone(&blacklist);

    walker.run(|| {
        let tx = tx.clone();
        let blacklist = Arc::clone(&blacklist);

        Box::new(move |result| {
            if let Ok(entry) = result {
                let path = entry.path().to_path_buf();

                if path.is_file()
                    && !blacklist.should_skip_path(&path)
                    && tx.blocking_send(path).is_err()
                {
                    return WalkState::Quit;
                }
            }
            WalkState::Continue
        })
    });
}

/// Starts the filesystem scan and the AI worker pool that processes behind it.
pub fn start_indexing(
    app_handle: tauri::AppHandle,
    blacklist: Arc<Blacklist>,
) -> Result<(), String> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<PathBuf>(1000);
    let (ai_tx, ai_rx) = tokio::sync::mpsc::channel::<i64>(AI_QUEUE_CAPACITY);

    let root_path = PathBuf::from("/");
    let blacklist = Arc::clone(&blacklist);

    let walker_blacklist = Arc::clone(&blacklist);
    tauri::async_runtime::spawn(async move {
        traverse_system(root_path, walker_blacklist, tx);
    });

    ai::start_ai_processing(
        app_handle.clone(),
        Arc::clone(&blacklist),
        ai_tx.clone(),
        ai_rx,
    );

    tauri::async_runtime::spawn(async move {
        while let Some(file_path) = rx.recv().await {
            // Every file the walker emits counts as indexed for this session,
            // even if it was already in the database from an earlier run.
            if let Some(usage) = app_handle.try_state::<Arc<UsageCounters>>() {
                usage.incr_files_indexed();
            }

            match process_file(&app_handle, &file_path).await {
                Ok(row_id) => {
                    if ai_tx.send(row_id).await.is_err() {
                        eprintln!(
                            "AI pipeline closed, dropped row {row_id} for {:?}",
                            file_path
                        );
                    }
                }
                Err(err) => eprintln!("Failed to process file {:?}: {}", file_path, err),
            }
        }
    });

    Ok(())
}
