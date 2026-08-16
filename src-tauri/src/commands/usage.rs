use std::sync::Arc;

use crate::services::usage::{UsageCounters, UsageSnapshot};

#[tauri::command]
#[specta::specta]
pub fn get_usage(state: tauri::State<Arc<UsageCounters>>) -> UsageSnapshot {
    state.snapshot()
}
