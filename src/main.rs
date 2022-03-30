mod byte_functions;
mod query;
mod db;

use actix_web::{get, App, HttpServer, Responder, web, HttpRequest, HttpResponse, http::StatusCode};
use rand::{thread_rng, prelude::SliceRandom};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

use serde::Deserialize;

// If not more than 31, possible not online
// So dont waste bandwidth on redis query etc.
const THIRTY_ONE_MINUTES: u64 = 60 * 31 * 1000;

#[derive(Debug, Deserialize)]
pub struct AnnounceRequest {
    infohash: String,
    port: u16,
}

#[get("/healthz")]
async fn healthz(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {    
   
    let query = req.query_string();
    println!("OG QUERY IS {}", query);
    let conn_info = req.connection_info();
    let user_ip = conn_info.peer_addr().expect("Missing IP bruv");

    let parsed =  match query::parse_announce(user_ip, query.replace("%", "%25").as_bytes()) {
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
        
    let time_now = SystemTime::now().duration_since(UNIX_EPOCH).expect("fucked up");
    let time_now_ms: u64 = u64::try_from(time_now.as_millis()).expect("fucc");

    // Get seeders & leechers
    let mut rc = data.redis_connection.clone();

    let seeders_key = parsed.info_hash.clone() + "_seeders";
    let leechers_key = parsed.info_hash.clone() + "_leechers";

    let (seeders, mut leechers) : (Vec<Vec<u8>>, Vec<Vec<u8>>) = redis::pipe()
    .cmd("ZRANGEBYSCORE").arg(&seeders_key).arg(time_now_ms - THIRTY_ONE_MINUTES).arg(time_now_ms)
    .cmd("ZRANGEBYSCORE").arg(&leechers_key).arg(time_now_ms - THIRTY_ONE_MINUTES).arg(time_now_ms)
    .query_async(&mut rc).await.unwrap();

    let is_seeder = seeders.contains(&parsed.ip_port);

    let is_leecher = match is_seeder {
        true => false, // If it's a seeder, leecher must be false
        false => leechers.contains(&parsed.ip_port), // otherwise we will search the leecher vector as well
    };

    // Don't shuffle the seeders, for leechers shuffle so that the older ones also get a shot
    // e.g. if there are 1000 leechers, the one whom announced 29 minutes ago also has a fair
    // change of being announced to a peer, to help swarm
    leechers.shuffle(&mut thread_rng());

    let mut post_announce_pipeline = redis::pipe();

    // These will contain how we change the total number of seeders / leechers by the end of the announce
    let mut seed_count_mod = 0;
    let mut leech_count_mod = 0;

    let event = match &parsed.event {
        Some(event_value) => event_value,
        None => "unknown",
    };

    // let bg_rc = data.redis_connection.clone();

    let final_response: HttpResponse = if event == "stopped" {
        if is_seeder {
            println!("Stopped and is seeder");
            seed_count_mod -= 1;
            post_announce_pipeline.cmd("ZREM").arg(&seeders_key).arg(&parsed.ip_port).ignore(); // We dont care about the return value
            HttpResponse::build(StatusCode::OK).body("TRUE TRUE\n")
        } else if is_leecher {
            println!("Stopped and is leecher");
            leech_count_mod -= 1;
            post_announce_pipeline.cmd("ZREM").arg(&leechers_key).arg(&parsed.ip_port).ignore(); // We dont care about the return value
            HttpResponse::build(StatusCode::OK).body("TRUE FALSE\n")
        } else {
            println!("NA, dodgy");
            HttpResponse::build(StatusCode::OK).body("FALSE FALSE\n")
        }
    } else if parsed.is_seeding {

        // New seeder
        if is_seeder == false {
            println!("Seeding now (new)");
            post_announce_pipeline.cmd("ZADD").arg(&seeders_key).arg(time_now_ms).arg(&parsed.ip_port).ignore();
            seed_count_mod += 1
        }

        // They just completed
        if event == "completed" {
            println!("Completed");    
            // If they were previously leecher, remove from that pool
            if is_leecher {
                println!("Removing as old leecher");
                post_announce_pipeline.cmd("ZREM").arg(&leechers_key).arg(&parsed.ip_port).ignore();
                leech_count_mod -= 1
            }

            // Increment the downloaded count for the infohash stats
            post_announce_pipeline.cmd("HINCRBY").arg(&parsed.info_hash).arg("downloaded").arg(1).ignore();
        }

        HttpResponse::build(StatusCode::OK).body("IS_SEEDING\n")
    } else {

        // New leecher
        if is_leecher == false {
            println!("New leecher");
            post_announce_pipeline.cmd("ZADD").arg(&leechers_key).arg(time_now_ms).arg(&parsed.ip_port).ignore();
            leech_count_mod += 1
        }

        HttpResponse::build(StatusCode::OK).body("IS_LEECHING\n")
    };

    // Update seeder & leecher count, if required
    if seed_count_mod != 0 {
        println!("Seed count mod: {}", seed_count_mod);
        post_announce_pipeline.cmd("HINCRBY").arg(&parsed.info_hash).arg("seeders").arg(seed_count_mod).ignore();
    }

    if leech_count_mod != 0 {
        println!("Leech `count mod: {}", leech_count_mod);
        post_announce_pipeline.cmd("HINCRBY").arg(&parsed.info_hash).arg("leechers").arg(leech_count_mod).ignore();
    }

    actix_web::rt::spawn(async move {
        // println!("GOING TO SLEEP");
        // actix_web::rt::time::sleep(Duration::from_millis(2000)).await;
        // println!("WOKE UP");
        let () = post_announce_pipeline.query_async(&mut rc).await.expect("Failed to execute pipe on redis");
    });

    println!("My IP_port combo is {:?}", &parsed.ip_port);
    println!("Seeders are {:?}", seeders);
    println!("Leechers are {:?}", leechers);
    println!("is_seeder: {}", is_seeder);
    println!("is_leecher: {}", is_leecher);

    let bruvva_res = query::announce_reply(seeders.len(), leechers.len(), &seeders[0..50], &leechers[0..50]);

    // return final_response;
    return HttpResponse::build(StatusCode::OK).body(bruvva_res);
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
    .bind(("0.0.0.0", 8888))?
    .run()
    .await;
}
