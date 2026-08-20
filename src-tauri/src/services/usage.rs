use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::atomic::{AtomicU64, Ordering};

/// Session-scoped counters. All increments use relaxed ordering: on a hot path
/// like indexing there is no need for cross-thread synchronization beyond the
/// atomics themselves.
#[derive(Debug, Default)]
pub struct UsageCounters {
    pub requests: AtomicU64,
    pub tokens: AtomicU64,
    pub files_indexed: AtomicU64,
    pub files_ai_indexed: AtomicU64,
}

impl UsageCounters {
    pub fn record_request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_tokens(&self, tokens: u64) {
        self.tokens.fetch_add(tokens, Ordering::Relaxed);
    }

    pub fn incr_files_indexed(&self) {
        self.files_indexed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn incr_files_ai_indexed(&self) {
        self.files_ai_indexed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> UsageSnapshot {
        UsageSnapshot {
            requests: self.requests.load(Ordering::Relaxed),
            tokens: self.tokens.load(Ordering::Relaxed),
            files_indexed: self.files_indexed.load(Ordering::Relaxed),
            files_ai_indexed: self.files_ai_indexed.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct UsageSnapshot {
    pub requests: u64,
    pub tokens: u64,
    pub files_indexed: u64,
    pub files_ai_indexed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_increment_and_snapshot() {
        let counters = UsageCounters::default();
        counters.record_request();
        counters.record_request();
        counters.add_tokens(42);
        counters.incr_files_indexed();
        counters.incr_files_ai_indexed();

        let snap = counters.snapshot();
        assert_eq!(snap.requests, 2);
        assert_eq!(snap.tokens, 42);
        assert_eq!(snap.files_indexed, 1);
        assert_eq!(snap.files_ai_indexed, 1);
    }

    #[test]
    fn snapshot_starts_at_zero() {
        let counters = UsageCounters::default();
        assert_eq!(
            counters.snapshot(),
            UsageSnapshot {
                requests: 0,
                tokens: 0,
                files_indexed: 0,
                files_ai_indexed: 0,
            }
        );
    }
}