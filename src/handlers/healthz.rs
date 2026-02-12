use actix_web::{get, http::{header, StatusCode}, web, HttpResponse};

use crate::AppState;

#[get("/healthz")]
pub async fn healthz(data: web::Data<AppState>) -> HttpResponse {
    let mut rc = data.redis_connection.clone();

    match crate::trace_wrap_v2!(redis::cmd("PING").query_async::<()>(&mut rc).await, "redis", "healthcheck") {
        Ok(_) => HttpResponse::build(StatusCode::OK)
            .append_header(header::ContentType::plaintext())
            .body("OK"),
        Err(_) => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .append_header(header::ContentType::plaintext())
            .body("OOF"),
    }
}

