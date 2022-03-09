mod byte_functions;

use actix_web::{get, App, HttpServer, Responder, web, HttpRequest};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AnnounceRequest {
    infohash: String,
    port: u16,
}

#[get("/healthz")]
async fn healthz(req: HttpRequest) -> impl Responder {
    let query = req.query_string();
    println!("Query string is {}", query);
    let conn_info = req.connection_info();
    let user_ip = conn_info.peer_addr().expect("Missing IP bruv");
    byte_functions::ip_str_to_bytes(user_ip);
    println!("Bruh {:?}", user_ip);
    return "";
}

#[get("/announce")]
async fn announce(params: web::Query<AnnounceRequest>) -> impl Responder {
    byte_functions::do_nothing();
    println!("Got the following params: {:?}", byte_functions::url_encoded_to_hex(&params.infohash));
    println!("Port reported as {}", params.port);
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
