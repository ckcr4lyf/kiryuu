use actix_web::{get, http::{header, StatusCode}, web, HttpResponse};

use crate::AppState;

#[get("/healthz")]
pub async fn healthz(data: web::Data<AppState>) -> HttpResponse {
    let mut rc = data.redis_connection.clone();

    match redis::cmd("PING").query_async::<()>(&mut rc).await {
        Ok(_) => {
            let active = data.active_requests.lock().unwrap();
            HttpResponse::build(StatusCode::OK)
                .append_header(header::ContentType::plaintext())
                .body(format!("OK\nactive_requests={}", active))
        }
        Err(_) => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .append_header(header::ContentType::plaintext())
            .body("OOF"),
    }
}

