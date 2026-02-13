use actix_web::{get, http::{header, StatusCode}, web, HttpResponse};

use crate::constants;
use crate::AppState;

const PROMETHEUS_LABELS: &str = r#"{status_code="200", method="GET", path="announce"}"#;

#[get("/metrics")]
pub async fn metrics(data: web::Data<AppState>) -> HttpResponse {
    let mut rc = data.redis_connection.clone();

    let (nochange, cache_hit, announce_count, req_duration): (
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
    ) = match redis::pipe()
        .get(constants::NOCHANGE_ANNOUNCE_COUNT_KEY)
        .get(constants::CACHE_HIT_ANNOUNCE_COUNT_KEY)
        .get(constants::ANNOUNCE_COUNT_KEY)
        .get(constants::REQ_DURATION_KEY)
        .query_async(&mut rc)
        .await
    {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .append_header(header::ContentType::plaintext())
                .body("redis error");
        }
    };

    let nochange = nochange.unwrap_or(0);
    let cache_hit = cache_hit.unwrap_or(0);
    let announce_count = announce_count.unwrap_or(0);
    let req_duration = req_duration.unwrap_or(0);

    let body = format!(
        "kouko_http_nochange_request_count{} {}\n\
         kouko_http_cache_hit_request_count{} {}\n\
         kouko_http_request_count{} {}\n\
         kouko_http_request_duration_sum{} {}\n",
        PROMETHEUS_LABELS,
        nochange,
        PROMETHEUS_LABELS,
        cache_hit,
        PROMETHEUS_LABELS,
        announce_count,
        PROMETHEUS_LABELS,
        req_duration,
    );

    HttpResponse::build(StatusCode::OK)
        .append_header(header::ContentType(
            "text/plain; version=0.0.4; charset=utf-8"
                .parse()
                .unwrap(),
        ))
        .body(body)
}
