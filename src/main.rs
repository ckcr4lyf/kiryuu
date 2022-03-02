mod byte_functions;

use actix_web::{get, App, HttpServer, Responder, web};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AnnounceRequest {
    infohash: String,
}

#[get("/healthz")]
async fn healthz() -> impl Responder {
    return "";
}

#[get("/announce")]
async fn announce(params: web::Query<AnnounceRequest>) -> impl Responder {
    byte_functions::do_nothing();
    println!("Got the following params: {:?}", params.infohash);
    return "GG";
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    return HttpServer::new(|| {
        App::new()
        .service(healthz)
        .service(announce)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await;
}
