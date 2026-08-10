use super::blacklist::should_skip_path;
use crate::services::indexer::blacklist::Blacklist;
use ignore::{WalkBuilder, WalkState};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::services::indexer::processing::process_file;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

pub fn traverse_system(root: PathBuf, blacklist: Arc<Blacklist>, tx: mpsc::Sender<PathBuf>) {
    // Arc is used to share the blacklist across threads without needing to clone it for each entry
    let filter_blacklist = Arc::clone(&blacklist);

    let walker = WalkBuilder::new(root)
        .threads(std::thread::available_parallelism().unwrap().get())
        .hidden(false) // Include hidden files
        .git_ignore(false)
        .same_file_system(true)
        // Skip blacklisted directories
        .filter_entry(move |entry| {
            !should_skip_path(&entry.path().to_path_buf(), &filter_blacklist)
        })
        .build_parallel();

    let blacklist = Arc::clone(&blacklist);

    walker.run(|| {
        let tx = tx.clone();
        let blacklist = Arc::clone(&blacklist);

        Box::new(move |result| {
            if let Ok(entry) = result {
                let path = entry.path().to_path_buf();

                if path.is_file()
                    && !should_skip_path(&path, &blacklist)
                    && tx.blocking_send(path).is_err()
                {
                    return WalkState::Quit;
                }
            }
            WalkState::Continue
        })
    });
}

pub fn start_indexing(
    app_handle: tauri::AppHandle,
    blacklist: Arc<Blacklist>,
) -> Result<(), String> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<PathBuf>(1000);

    let root_path = PathBuf::from("/");
    let blacklist = Arc::clone(&blacklist);

    tauri::async_runtime::spawn(async move {
        traverse_system(root_path, blacklist, tx);
    });

    tauri::async_runtime::spawn(async move {
        while let Some(file_path) = rx.recv().await {
            if let Err(err) = process_file(&app_handle, &file_path).await {
                eprintln!("Failed to process file {:?}: {}", file_path, err);
            }
        }
    });

    Ok(())
}
