mod byte_functions;
mod query;
mod db;

use actix_web::{get, App, HttpServer, Responder, web, HttpRequest, HttpResponse, http::StatusCode};
// use redis::Commands;
use redis::AsyncCommands;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AnnounceRequest {
    infohash: String,
    port: u16,
}

#[get("/healthz")]
async fn healthz(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {    
   
    let query = req.query_string();
    let conn_info = req.connection_info();
    let user_ip = conn_info.peer_addr().expect("Missing IP bruv");

    let parsed =  match query::parse_announce(user_ip, query) {
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

    // Get seeders
    let mut rc = data.redis_connection.clone();
    let seeders: Vec<Vec<u8>> = rc.zrangebyscore(parsed.info_hash + "_seeders", 0u64, 2_648_029_777_853u64).await.unwrap();
    println!("Seeders are {:?}", seeders);

    // println!("Peer info: {:?}", parsed);
    return HttpResponse::build(StatusCode::OK).body("OK\n");
}

#[get("/announce")]
async fn announce(params: web::Query<AnnounceRequest>) -> impl Responder {
    byte_functions::do_nothing();
    println!("Got the following params: {:?}", byte_functions::url_encoded_to_hex(&params.infohash));
    println!("Port reported as {}", params.port);
    return "GG";
}

// #[derive(Debug)]
struct AppState {
    redis_connection: redis::aio::MultiplexedConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let redis = redis::Client::open("redis://127.0.0.1").unwrap();
    let redis_connection = redis.get_multiplexed_tokio_connection().await.unwrap();

    let data = web::Data::new(AppState{
        redis_connection: redis_connection,
    });

    return HttpServer::new(move || {
        App::new()
        .app_data(data.clone())
        .service(healthz)
        .service(announce)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await;
}
