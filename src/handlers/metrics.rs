use actix_web::{get, http::{header, StatusCode}, web, HttpResponse};
use tikv_jemalloc_ctl::{epoch, stats};

use crate::byte_functions::overlong_infohash_count;
use crate::store::HISTOGRAM_BUCKET_UPPER_BOUNDS_SECS;
use crate::AppState;

const PROMETHEUS_LABELS: &str = r#"{status_code="200", method="GET", path="announce"}"#;

#[get("/metrics")]
pub async fn metrics(data: web::Data<AppState>) -> HttpResponse {
    let (nochange, cache_hit, announce_count, req_duration_ms, req_duration_us) =
        data.store.stats.snapshot();
    let (histogram_sum_us, histogram_count, histogram_buckets) =
        data.store.stats.histogram_snapshot();
    let active_requests = *data.active_requests.lock();
    let (torrent_count, seeder_count, leecher_count) = data.store.peer_totals();
    let peer_count = seeder_count + leecher_count;
    let sweep = data.store.stats.sweep_snapshot();

    let (resident_bytes, allocated_bytes) = (|| {
        epoch::advance().ok()?;
        let resident = stats::resident::read().ok()?;
        let allocated = stats::allocated::read().ok()?;
        Some((resident, allocated))
    })()
    .unwrap_or((0, 0));

    let mut body = format!(
        "kiryuu_http_nochange_request_count{} {}\n\
         kiryuu_http_cache_hit_request_count{} {}\n\
         kiryuu_http_request_count{} {}\n\
         kiryuu_http_request_duration_sum{} {}\n\
         kiryuu_http_request_duration_sum_us{} {}\n\
         kiryuu_active_request_count {}\n\
         kiryuu_torrent_count {}\n\
         kiryuu_peer_count {}\n\
         kiryuu_seeder_count {}\n\
         kiryuu_leecher_count {}\n\
         kiryuu_resident_bytes {}\n\
         kiryuu_allocated_bytes {}\n\
         kiryuu_stripe_index_size {}\n\
         kiryuu_stripe_index_repaired {}\n\
         kiryuu_sweep_duration_seconds {}\n\
         kiryuu_sweep_duration_seconds_sum {}\n\
         kiryuu_sweep_count {}\n\
         kiryuu_sweep_visited {}\n\
         kiryuu_sweep_removed {}\n\
         kiryuu_sweep_orphans_removed {}\n\
         kiryuu_peer_totals_refresh_duration_seconds {}\n\
         kiryuu_malformed_infohash_count {}\n",
        PROMETHEUS_LABELS,
        nochange,
        PROMETHEUS_LABELS,
        cache_hit,
        PROMETHEUS_LABELS,
        announce_count,
        PROMETHEUS_LABELS,
        req_duration_ms,
        PROMETHEUS_LABELS,
        req_duration_us,
        active_requests,
        torrent_count,
        peer_count,
        seeder_count,
        leecher_count,
        resident_bytes,
        allocated_bytes,
        data.store.stripe_index_size(),
        sweep.index_repaired,
        sweep.last_duration_us as f64 / 1_000_000.0,
        sweep.duration_sum_us as f64 / 1_000_000.0,
        sweep.count,
        sweep.visited,
        sweep.removed,
        sweep.orphans_removed,
        sweep.totals_refresh_last_us as f64 / 1_000_000.0,
        overlong_infohash_count(),
    );

    for (bucket_count, upper_bound) in histogram_buckets
        .iter()
        .zip(HISTOGRAM_BUCKET_UPPER_BOUNDS_SECS.iter())
    {
        body.push_str(&format!(
            "kiryuu_http_request_duration_histogram_bucket{{status_code=\"200\", method=\"GET\", path=\"announce\", le=\"{upper_bound}\"}} {bucket_count}\n"
        ));
    }
    body.push_str(&format!(
        "kiryuu_http_request_duration_histogram_bucket{{status_code=\"200\", method=\"GET\", path=\"announce\", le=\"+Inf\"}} {histogram_count}\n"
    ));
    body.push_str(&format!(
        "kiryuu_http_request_duration_histogram_sum{{status_code=\"200\", method=\"GET\", path=\"announce\"}} {}\n",
        histogram_sum_us as f64 / 1_000_000.0
    ));
    body.push_str(&format!(
        "kiryuu_http_request_duration_histogram_count{{status_code=\"200\", method=\"GET\", path=\"announce\"}} {histogram_count}\n"
    ));

    HttpResponse::build(StatusCode::OK)
        .append_header(header::ContentType(
            "text/plain; version=0.0.4; charset=utf-8"
                .parse()
                .unwrap(),
        ))
        .body(body)
}
