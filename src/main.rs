mod byte_functions;
mod query;
mod db;

use actix_web::{get, App, HttpServer, Responder, web, HttpRequest, HttpResponse, http::StatusCode};
use redis::AsyncCommands;
// use redis::AsyncCommands;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AnnounceRequest {
    infohash: String,
    port: u16,
}

#[get("/healthz")]
async fn healthz(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {    

    db::get_guys();

    // req.app_data()
    // let x = req.app_data::<AppState>().unwrap();

    // let z = req.app_data::<AppState>().unwrap();

    // &z.redis_connection.zrembylex("XD", 4u32, 6u32);




    let y: i32 = 100;

    // let guys: Vec<Vec<u8>> = (*x).redis_connection.zrangebyscore("abc", y, y).await.unwrap();

    // let bruvva = req.app_data::<AppState>().unwrap();

    println!("App data is {:?}", data);
    


    
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


    println!("Peer info: {:?}", parsed);
    return HttpResponse::build(StatusCode::OK).body("OK\n");
}

#[get("/announce")]
async fn announce(params: web::Query<AnnounceRequest>) -> impl Responder {
    byte_functions::do_nothing();
    println!("Got the following params: {:?}", byte_functions::url_encoded_to_hex(&params.infohash));
    println!("Port reported as {}", params.port);
    return "GG";
}

#[derive(Debug)]
struct AppState {
    // redis_connection: redis::aio::MultiplexedConnection,
    // redis_connection: redis::aio::ConnectionManager,
    bruv: u32,
    // redis_connection: redis::Connection,
    // xd: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // let redis = redis::Client::open("redis://127.0.0.1").unwrap();
    // let redis_connection = redis.get_tokio_connection_manager().await.unwrap();
    // let redis_connection = redis.get_connection().unwrap();

    return HttpServer::new(|| {
        App::new()
        .app_data(web::Data::new(AppState{
                // redis_connection: redis_connection.clone(),
                // redis_connection: redis_connection.clone(),

                bruv: 69,
        }))
        .service(healthz)
        .service(announce)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await;
}
