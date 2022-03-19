mod byte_functions;
mod query;

use actix_web::{get, App, HttpServer, Responder, web, HttpRequest, HttpResponse, http::StatusCode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AnnounceRequest {
    infohash: String,
    port: u16,
}

#[get("/healthz")]
async fn healthz(req: HttpRequest) -> HttpResponse {    
    let query = req.query_string();

    let parsed =  match query::parse_announce(query) {
        Ok(legit) => legit, // Just set `parsed` , let handler continue
        Err(e) => match e {
            query::QueryError::ParseFailure => {
                return HttpResponse::build(StatusCode::BAD_REQUEST).body("Failed to parse announce\n");
            }
            query::QueryError::Custom(e) => {
                return HttpResponse::build(StatusCode::BAD_REQUEST).body(e + "\n");
            }
        }
    };

    let conn_info = req.connection_info();
    let user_ip = conn_info.peer_addr().expect("Missing IP bruv");

    byte_functions::ip_str_port_u16_to_bytes(user_ip, parsed.port);

    return HttpResponse::build(StatusCode::OK).body("OK\n");
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
