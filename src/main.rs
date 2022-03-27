mod byte_functions;
mod query;
mod db;

use actix_web::{get, App, HttpServer, Responder, web, HttpRequest, HttpResponse, http::StatusCode};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

const TWO_HOURS: u64 = 60 * 60 * 2 * 1000;

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

        
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).expect("fucked up");
    let since_epoch_ms: u64 = u64::try_from(since_epoch.as_millis()).expect("fucc");

    // Get seeders & leechers
    let mut rc = data.redis_connection.clone();

    let (seeders, leechers) : (Vec<Vec<u8>>, Vec<Vec<u8>>) = redis::pipe()
    .cmd("ZRANGEBYSCORE").arg(parsed.info_hash.clone() + "_seeders").arg(since_epoch_ms - TWO_HOURS).arg(since_epoch_ms)
    .cmd("ZRANGEBYSCORE").arg(parsed.info_hash.clone() + "_leechers").arg(since_epoch_ms - TWO_HOURS).arg(since_epoch_ms)
    .query_async(&mut rc).await.unwrap();


    println!("Range is {} {}", since_epoch_ms - TWO_HOURS, since_epoch_ms);

    let is_seeder = seeders.contains(&parsed.ip_port);

    let is_leecher = match is_seeder {
        true => false, // If it's a seeder, leecher must be false
        false => leechers.contains(&parsed.ip_port), // otherwise we will search the next vector as well
    };
    
    println!("My IP_port combo is {:?}", &parsed.ip_port);
    println!("Seeders are {:?}", seeders);
    println!("Leechers are {:?}", leechers);
    println!("is_seeder: {}", is_seeder);
    println!("is_leecher: {}", is_leecher);

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
