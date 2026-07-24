use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct TrackerStats {
    pub announce_count: AtomicU64,
    pub nochange_count: AtomicU64,
    pub cache_hit_count: AtomicU64,
    pub req_duration_sum_ms: AtomicI64,
}

impl TrackerStats {
    pub fn record_announce(&self, req_duration_ms: i64) {
        self.announce_count.fetch_add(1, Ordering::Relaxed);
        self.req_duration_sum_ms
            .fetch_add(req_duration_ms, Ordering::Relaxed);
    }

    pub fn record_cache_hit(&self) {
        self.cache_hit_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_nochange(&self) {
        self.nochange_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> (u64, u64, u64, i64) {
        (
            self.nochange_count.load(Ordering::Relaxed),
            self.cache_hit_count.load(Ordering::Relaxed),
            self.announce_count.load(Ordering::Relaxed),
            self.req_duration_sum_ms.load(Ordering::Relaxed),
        )
    }
}
