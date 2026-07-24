use actix_web::{get, http::{header, StatusCode}, web, HttpResponse};
use tikv_jemalloc_ctl::{epoch, stats};

use crate::AppState;

const PROMETHEUS_LABELS: &str = r#"{status_code="200", method="GET", path="announce"}"#;

#[get("/metrics")]
pub async fn metrics(data: web::Data<AppState>) -> HttpResponse {
    let (nochange, cache_hit, announce_count, req_duration) = data.store.stats.snapshot();
    let active_requests = *data.active_requests.lock();
    let (torrent_count, seeder_count, leecher_count) = data.store.peer_totals();
    let peer_count = seeder_count + leecher_count;

    let (resident_bytes, allocated_bytes) = (|| {
        epoch::advance().ok()?;
        let resident = stats::resident::read().ok()?;
        let allocated = stats::allocated::read().ok()?;
        Some((resident, allocated))
    })()
    .unwrap_or((0, 0));

    let body = format!(
        "kiryuu_http_nochange_request_count{} {}\n\
         kiryuu_http_cache_hit_request_count{} {}\n\
         kiryuu_http_request_count{} {}\n\
         kiryuu_http_request_duration_sum{} {}\n\
         kiryuu_active_request_count {}\n\
         kiryuu_torrent_count {}\n\
         kiryuu_peer_count {}\n\
         kiryuu_seeder_count {}\n\
         kiryuu_leecher_count {}\n\
         kiryuu_resident_bytes {}\n\
         kiryuu_allocated_bytes {}\n",
        PROMETHEUS_LABELS,
        nochange,
        PROMETHEUS_LABELS,
        cache_hit,
        PROMETHEUS_LABELS,
        announce_count,
        PROMETHEUS_LABELS,
        req_duration,
        active_requests,
        torrent_count,
        peer_count,
        seeder_count,
        leecher_count,
        resident_bytes,
        allocated_bytes,
    );

    HttpResponse::build(StatusCode::OK)
        .append_header(header::ContentType(
            "text/plain; version=0.0.4; charset=utf-8"
                .parse()
                .unwrap(),
        ))
        .body(body)
}
