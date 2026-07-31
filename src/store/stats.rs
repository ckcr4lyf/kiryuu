use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::Duration;

pub const HISTOGRAM_BUCKET_UPPER_BOUNDS_SECS: [f64; 17] = [
    0.00005, 0.0001, 0.00025, 0.0005, 0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5,
    1.0, 2.5, 5.0, 10.0,
];

#[derive(Debug)]
pub struct TrackerStats {
    pub announce_count: AtomicU64,
    pub nochange_count: AtomicU64,
    pub cache_hit_count: AtomicU64,
    pub req_duration_sum_ms: AtomicI64,
    pub req_duration_sum_us: AtomicI64,
    histogram_buckets: [AtomicU64; HISTOGRAM_BUCKET_UPPER_BOUNDS_SECS.len()],
    histogram_sum_us: AtomicU64,
    histogram_count: AtomicU64,
    /// GC-thread sweep accounting. Written only by the `kiryuu-gc` thread.
    sweep_last_us: AtomicU64,
    sweep_sum_us: AtomicU64,
    sweep_count: AtomicU64,
    sweep_visited: AtomicU64,
    sweep_removed: AtomicU64,
    sweep_orphans_removed: AtomicU64,
    stripe_index_repaired: AtomicU64,
    totals_refresh_last_us: AtomicU64,
}

/// Snapshot of GC sweep counters for `/metrics`.
pub struct SweepStats {
    pub last_duration_us: u64,
    pub duration_sum_us: u64,
    pub count: u64,
    pub visited: u64,
    pub removed: u64,
    pub orphans_removed: u64,
    pub index_repaired: u64,
    pub totals_refresh_last_us: u64,
}

impl Default for TrackerStats {
    fn default() -> Self {
        Self {
            announce_count: AtomicU64::new(0),
            nochange_count: AtomicU64::new(0),
            cache_hit_count: AtomicU64::new(0),
            req_duration_sum_ms: AtomicI64::new(0),
            req_duration_sum_us: AtomicI64::new(0),
            histogram_buckets: std::array::from_fn(|_| AtomicU64::new(0)),
            histogram_sum_us: AtomicU64::new(0),
            histogram_count: AtomicU64::new(0),
            sweep_last_us: AtomicU64::new(0),
            sweep_sum_us: AtomicU64::new(0),
            sweep_count: AtomicU64::new(0),
            sweep_visited: AtomicU64::new(0),
            sweep_removed: AtomicU64::new(0),
            sweep_orphans_removed: AtomicU64::new(0),
            stripe_index_repaired: AtomicU64::new(0),
            totals_refresh_last_us: AtomicU64::new(0),
        }
    }
}

impl TrackerStats {
    pub fn record_announce(&self, duration: Duration) {
        let us = i64::try_from(duration.as_micros()).unwrap_or(i64::MAX);
        let ms = i64::try_from(duration.as_millis()).unwrap_or(i64::MAX);

        self.announce_count.fetch_add(1, Ordering::Relaxed);
        self.req_duration_sum_ms.fetch_add(ms, Ordering::Relaxed);
        self.req_duration_sum_us.fetch_add(us, Ordering::Relaxed);
        self.record_histogram(duration);
    }

    fn record_histogram(&self, duration: Duration) {
        let secs = duration.as_secs_f64();
        let us = u64::try_from(duration.as_micros()).unwrap_or(u64::MAX);

        self.histogram_sum_us.fetch_add(us, Ordering::Relaxed);
        self.histogram_count.fetch_add(1, Ordering::Relaxed);

        for (bucket, upper_bound) in self
            .histogram_buckets
            .iter()
            .zip(HISTOGRAM_BUCKET_UPPER_BOUNDS_SECS.iter())
        {
            if secs <= *upper_bound {
                bucket.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn record_cache_hit(&self) {
        self.cache_hit_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_nochange(&self) {
        self.nochange_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_sweep(&self, duration: Duration, visited: usize, removed: usize, orphans: usize) {
        let us = u64::try_from(duration.as_micros()).unwrap_or(u64::MAX);

        self.sweep_last_us.store(us, Ordering::Relaxed);
        self.sweep_sum_us.fetch_add(us, Ordering::Relaxed);
        self.sweep_count.fetch_add(1, Ordering::Relaxed);
        self.sweep_visited
            .fetch_add(visited as u64, Ordering::Relaxed);
        self.sweep_removed
            .fetch_add(removed as u64, Ordering::Relaxed);
        self.sweep_orphans_removed
            .fetch_add(orphans as u64, Ordering::Relaxed);
    }

    pub fn record_totals_refresh(&self, duration: Duration) {
        let us = u64::try_from(duration.as_micros()).unwrap_or(u64::MAX);
        self.totals_refresh_last_us.store(us, Ordering::Relaxed);
    }

    pub fn record_index_repair(&self, repaired: usize) {
        self.stripe_index_repaired
            .fetch_add(repaired as u64, Ordering::Relaxed);
    }

    pub fn sweep_snapshot(&self) -> SweepStats {
        SweepStats {
            last_duration_us: self.sweep_last_us.load(Ordering::Relaxed),
            duration_sum_us: self.sweep_sum_us.load(Ordering::Relaxed),
            count: self.sweep_count.load(Ordering::Relaxed),
            visited: self.sweep_visited.load(Ordering::Relaxed),
            removed: self.sweep_removed.load(Ordering::Relaxed),
            orphans_removed: self.sweep_orphans_removed.load(Ordering::Relaxed),
            index_repaired: self.stripe_index_repaired.load(Ordering::Relaxed),
            totals_refresh_last_us: self.totals_refresh_last_us.load(Ordering::Relaxed),
        }
    }

    pub fn snapshot(&self) -> (u64, u64, u64, i64, i64) {
        (
            self.nochange_count.load(Ordering::Relaxed),
            self.cache_hit_count.load(Ordering::Relaxed),
            self.announce_count.load(Ordering::Relaxed),
            self.req_duration_sum_ms.load(Ordering::Relaxed),
            self.req_duration_sum_us.load(Ordering::Relaxed),
        )
    }

    pub fn histogram_snapshot(&self) -> (u64, u64, [u64; HISTOGRAM_BUCKET_UPPER_BOUNDS_SECS.len()]) {
        let mut buckets = [0u64; HISTOGRAM_BUCKET_UPPER_BOUNDS_SECS.len()];
        for (out, bucket) in buckets.iter_mut().zip(self.histogram_buckets.iter()) {
            *out = bucket.load(Ordering::Relaxed);
        }

        (
            self.histogram_sum_us.load(Ordering::Relaxed),
            self.histogram_count.load(Ordering::Relaxed),
            buckets,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_microseconds_and_histogram_buckets() {
        let stats = TrackerStats::default();
        stats.record_announce(Duration::from_micros(200));

        assert_eq!(stats.req_duration_sum_us.load(Ordering::Relaxed), 200);
        assert_eq!(stats.histogram_count.load(Ordering::Relaxed), 1);

        let (_, count, buckets) = stats.histogram_snapshot();
        assert_eq!(count, 1);
        assert!(buckets[2] >= 1); // le=0.00025 (200µs)
    }
}
