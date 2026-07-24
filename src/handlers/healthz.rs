use actix_web::{get, http::{header, StatusCode}, web, HttpResponse};

use crate::AppState;

#[get("/healthz")]
pub async fn healthz(data: web::Data<AppState>) -> HttpResponse {
    let active = *data.active_requests.lock();
    HttpResponse::build(StatusCode::OK)
        .append_header(header::ContentType::plaintext())
        .body(format!("OK\nactive_requests={active}\ntorrents={}", data.store.torrent_count()))
}
