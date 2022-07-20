mod byte_functions;
mod query;
mod constants;
mod req_log;

use actix_web::{get, App, HttpServer, web, HttpRequest, HttpResponse, http::header, http::StatusCode};
use rand::{thread_rng, prelude::SliceRandom, Rng};
use std::time::{SystemTime, UNIX_EPOCH};

// If not more than 31, possible not online
// So dont waste bandwidth on redis query etc.
const THIRTY_ONE_MINUTES: i64 = 60 * 31 * 1000;

#[get("/announce")]
async fn announce(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {    
   
    let time_now = SystemTime::now().duration_since(UNIX_EPOCH).expect("fucked up");
    let time_now_ms: i64 = i64::try_from(time_now.as_millis()).expect("fucc");
    let max_limit = time_now_ms - THIRTY_ONE_MINUTES;

    let query = req.query_string();
    let conn_info = req.connection_info();

    let user_ip = match conn_info.peer_addr() {
        // We need to use this in pour async function after req is dropped
        // so we need to use `.to_owned()`
        Some(ip) => ip,
        None => {
            return HttpResponse::build(StatusCode::BAD_REQUEST).body("Missing IP");
        }
    };

    let user_ip_owned = user_ip.to_owned();

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

    // Get seeders & leechers
    let mut rc = data.redis_connection.clone();

    let seeders_key = parsed.info_hash.clone() + "_seeders";
    let leechers_key = parsed.info_hash.clone() + "_leechers";

    let (seeders, mut leechers) : (Vec<Vec<u8>>, Vec<Vec<u8>>) = redis::pipe()
    .cmd("ZRANGEBYSCORE").arg(&seeders_key).arg(max_limit).arg(time_now_ms)
    .cmd("ZRANGEBYSCORE").arg(&leechers_key).arg(max_limit).arg(time_now_ms)
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
    post_announce_pipeline.cmd("ZADD").arg(constants::TORRENTS_KEY).arg(time_now_ms).arg(&parsed.info_hash).ignore(); // To "update" the torrent

    // These will contain how we change the total number of seeders / leechers by the end of the announce
    let mut seed_count_mod: i64 = 0;
    let mut leech_count_mod: i64 = 0;

    let event = match &parsed.event {
        Some(event_value) => event_value,
        None => "unknown",
    };

    // let bg_rc = data.redis_connection.clone();

    if event == "stopped" {
        if is_seeder {
            seed_count_mod -= 1;
            post_announce_pipeline.cmd("ZREM").arg(&seeders_key).arg(&parsed.ip_port).ignore(); // We dont care about the return value
        } else if is_leecher {
            leech_count_mod -= 1;
            post_announce_pipeline.cmd("ZREM").arg(&leechers_key).arg(&parsed.ip_port).ignore(); // We dont care about the return value
        }
    } else if parsed.is_seeding {

        // New seeder
        if is_seeder == false {
            post_announce_pipeline.cmd("ZADD").arg(&seeders_key).arg(time_now_ms).arg(&parsed.ip_port).ignore();
            seed_count_mod += 1
        }

        // They just completed
        if event == "completed" {
            // If they were previously leecher, remove from that pool
            if is_leecher {
                post_announce_pipeline.cmd("ZREM").arg(&leechers_key).arg(&parsed.ip_port).ignore();
                leech_count_mod -= 1
            }

            // Increment the downloaded count for the infohash stats
            post_announce_pipeline.cmd("HINCRBY").arg(&parsed.info_hash).arg("downloaded").arg(1u32).ignore();
        }
    } else if is_leecher == false {
            post_announce_pipeline.cmd("ZADD").arg(&leechers_key).arg(time_now_ms).arg(&parsed.ip_port).ignore();
            leech_count_mod += 1
    };

    // Update seeder & leecher count, if required
    if seed_count_mod != 0 {
        post_announce_pipeline.cmd("HINCRBY").arg(&parsed.info_hash).arg("seeders").arg(seed_count_mod).ignore();
    }

    if leech_count_mod != 0 {
        post_announce_pipeline.cmd("HINCRBY").arg(&parsed.info_hash).arg("leechers").arg(leech_count_mod).ignore();
    }

    // endex = end index XD. seems in rust cannot select first 50 elements, or limit to less if vector doesnt have 50
    // e.g. &seeders[0..50] is panicking when seeders len is < 50. Oh well.
    let seeder_endex = if seeders.len() >= 50 {
        50
    } else {
        seeders.len()
    };

    let leecher_endex = if leechers.len() >= 50 {
        50
    } else {
        leechers.len()
    };

    let scount: i64 = seeders.len().try_into().expect("fucky wucky");
    let lcount: i64 = leechers.len().try_into().expect("fucky wucky");

    let bruvva_res = query::announce_reply(scount + seed_count_mod, lcount + leech_count_mod, &seeders[0..seeder_endex], &leechers[0..leecher_endex]);

    let time_end = SystemTime::now().duration_since(UNIX_EPOCH).expect("fucked up");
    let time_end_ms: i64 = i64::try_from(time_end.as_millis()).expect("fucc");

    let req_duration = time_end_ms - time_now_ms;

    post_announce_pipeline.cmd("INCR").arg(constants::ANNOUNCE_COUNT_KEY).ignore();
    post_announce_pipeline.cmd("INCRBY").arg(constants::REQ_DURATION_KEY).arg(req_duration).ignore();

    actix_web::rt::spawn(async move {
        // 0.1% chance to trigger a clean, 
        let chance = rand::thread_rng().gen_range(0..1000);
        if chance == 0 {
            post_announce_pipeline.cmd("ZREMRANGEBYSCORE").arg(&seeders_key).arg(0).arg(max_limit).ignore();
            post_announce_pipeline.cmd("ZREMRANGEBYSCORE").arg(&leechers_key).arg(0).arg(max_limit).ignore();
        }

        // log the summary
        post_announce_pipeline.cmd("PUBLISH").arg("reqlog").arg(req_log::generate_csv(&user_ip_owned, &parsed.info_hash)).ignore();


        let () = match post_announce_pipeline.query_async::<redis::aio::MultiplexedConnection, ()>(&mut rc).await {
            Ok(_) => (),
            Err(e) => {
                println!("Err during pipe {}. Timenow: {}, scountmod: {}, lcountmod: {}", e, time_now_ms, seed_count_mod, leech_count_mod);
                ()
            },
        };
    });

    return HttpResponse::build(StatusCode::OK).append_header(header::ContentType::plaintext()).body(bruvva_res);
}

#[get("/healthz")]
async fn healthz(data: web::Data<AppState>) -> HttpResponse {
    let mut rc = data.redis_connection.clone();
    let () = match redis::cmd("PING").query_async::<redis::aio::MultiplexedConnection, ()>(&mut rc).await {
        Ok(_) => {
            return HttpResponse::build(StatusCode::OK).append_header(header::ContentType::plaintext()).body("OK");
        },
        Err(_) => {
            return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).append_header(header::ContentType::plaintext()).body("FUCKED");
        }
    };
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
        redis_connection,
    });

    return HttpServer::new(move || {
        App::new()
        .app_data(data.clone())
        .service(healthz)
        .service(announce)
    })
    .bind(("0.0.0.0", 6969))?
    .max_connection_rate(8192)
    .keep_alive(None)
    .run()
    .await;
}
