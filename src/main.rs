mod byte_functions;
mod query;
mod constants;
mod req_log;

use actix_web::{get, App, HttpServer, web, HttpRequest, HttpResponse, http::header, http::StatusCode};
use std::{time::{SystemTime, UNIX_EPOCH}};
use clap::Parser;

/// Simple
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port for tracker to listen on. Default: 6969
    #[arg(long)]
    port: Option<u16>,

    /// IP to bind tracker to. Default: 0.0.0.0
    #[arg(long)]
    host: Option<String>,

    /// Address of redis instance. Default: 127.0.0.1:6379
    #[arg(long)]
    redis_host: Option<String>
}

// If not more than 31, possible not online
// So dont waste bandwidth on redis query etc.
const THIRTY_ONE_MINUTES: i64 = 60 * 31 * 1000;

#[derive(Debug)]
enum Exists {
    Yes,
    No,
}

impl redis::FromRedisValue for Exists {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Exists> {
        match *v {
            redis::Value::Nil => Ok(Exists::No),
            _ => Ok(Exists::Yes),
        }
    }
}

#[get("/announce")]
async fn announce(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {    
   
    let time_now = SystemTime::now().duration_since(UNIX_EPOCH).expect("fucked up");
    let time_now_ms: i64 = i64::try_from(time_now.as_millis()).expect("fucc");
    let max_limit = time_now_ms - THIRTY_ONE_MINUTES;

    let query = req.query_string();
    let peer_addr = req.peer_addr();

    let user_ip = if let Some(ref addr) = peer_addr {
        match addr {
            std::net::SocketAddr::V4(ref v4_addr) => v4_addr.ip(),
            _ => return HttpResponse::build(StatusCode::BAD_REQUEST).body("IPv6 not supported")
        }
    } else {
        return HttpResponse::build(StatusCode::BAD_REQUEST).body("Missing IP")
    };

    let parsed =  match query::parse_announce(user_ip, query.replace("%", "%25").as_bytes()) {
        Ok(legit) => legit, // Just set `parsed` , let handler continue
        Err(e) => match e {
            query::QueryError::ParseFailure => {
                return HttpResponse::build(StatusCode::BAD_REQUEST).body("Failed to parse announce\n");
            },
            query::QueryError::InvalidInfohash => {
                return HttpResponse::build(StatusCode::BAD_REQUEST).body("Infohash is not 20 bytes\n");
            }
        }
    };

    // Get seeders & leechers
    let mut rc = data.redis_connection.clone();
    let (seeders_key, leechers_key, cache_key) = byte_functions::make_redis_keys(&parsed.info_hash);

    let (is_seeder_v2, is_leecher_v2, cached_reply) : (Exists, Exists, Vec<u8>) = redis::pipe()
        .cmd("ZSCORE").arg(&seeders_key).arg(&parsed.ip_port)
        .cmd("ZSCORE").arg(&leechers_key).arg(&parsed.ip_port)
        .cmd("GET").arg(&cache_key)
        .query_async(&mut rc).await.unwrap();

    let mut post_announce_pipeline = redis::pipe();
    post_announce_pipeline.cmd("ZADD").arg(constants::TORRENTS_KEY).arg(time_now_ms).arg(&parsed.info_hash).ignore(); // To "update" the torrent

    // These will contain how we change the total number of seeders / leechers by the end of the announce
    let mut seed_count_mod: i64 = 0;
    let mut leech_count_mod: i64 = 0;


    if let query::Event::Stopped = parsed.event {
        if let Exists::Yes = is_seeder_v2 {
            seed_count_mod -= 1;
            post_announce_pipeline.cmd("ZREM").arg(&seeders_key).arg(&parsed.ip_port).ignore(); // We dont care about the return value
        } else if let Exists::Yes = is_leecher_v2 {
            leech_count_mod -= 1;
            post_announce_pipeline.cmd("ZREM").arg(&leechers_key).arg(&parsed.ip_port).ignore(); // We dont care about the return value
        }
    } else if parsed.is_seeding {

        // New seeder
        if let Exists::No = is_seeder_v2 {
            post_announce_pipeline.cmd("ZADD").arg(&seeders_key).arg(time_now_ms).arg(&parsed.ip_port).ignore();
            seed_count_mod += 1;
        }

        // They just completed
        if let query::Event::Completed = parsed.event {
            // If they were previously leecher, remove from that pool
            if let Exists::Yes = is_leecher_v2 {
                post_announce_pipeline.cmd("ZREM").arg(&leechers_key).arg(&parsed.ip_port).ignore();
                leech_count_mod -= 1
            }

            // Increment the downloaded count for the infohash stats
            post_announce_pipeline.cmd("HINCRBY").arg(&parsed.info_hash).arg("downloaded").arg(1u32).ignore();
        }
    } else if let Exists::No = is_leecher_v2 {
            post_announce_pipeline.cmd("ZADD").arg(&leechers_key).arg(time_now_ms).arg(&parsed.ip_port).ignore();
            leech_count_mod += 1;
    };

    // Cache miss = query redis
    // no change = update cache
    // change = clear cache

    let final_res = match cached_reply.len() {
        0 => {
            // Cache miss. Lookup from redis
            let (seeders, leechers) : (Vec<Vec<u8>>, Vec<Vec<u8>>) = redis::pipe()
            .cmd("ZRANGEBYSCORE").arg(&seeders_key).arg(max_limit).arg(time_now_ms).arg("LIMIT").arg(0).arg(50)
            .cmd("ZRANGEBYSCORE").arg(&leechers_key).arg(max_limit).arg(time_now_ms).arg("LIMIT").arg(0).arg(50)
            .query_async(&mut rc).await.unwrap();
        
            // endex = end index XD. seems in rust cannot select first 50 elements, or limit to less if vector doesnt have 50
            // e.g. &seeders[0..50] is panicking when seeders len is < 50. Oh well.
            let seeder_endex = std::cmp::min(seeders.len(), 50);
            let leecher_endex = std::cmp::min(leechers.len(), 50);

            query::announce_reply(seeders.len() as i64 + seed_count_mod, leechers.len() as i64 + leech_count_mod, &seeders[0..seeder_endex], &leechers[0..leecher_endex])
        },
        _ => {
            post_announce_pipeline.cmd("INCR").arg(constants::CACHE_HIT_ANNOUNCE_COUNT_KEY).ignore();
            cached_reply
        }
    };

    // Is there a change in seeders / leechers
    if seed_count_mod != 0 || leech_count_mod != 0 {
        // TBD: Maybe we can issue the HINCRBY anyway, it is:
        // Pipelined
        // In background (not .awaited for announce reply)
        // O(1) in redis
        // Can clean up this branching crap
        if seed_count_mod != 0 {
            post_announce_pipeline.cmd("HINCRBY").arg(&parsed.info_hash).arg("seeders").arg(seed_count_mod).ignore();
        }

        if leech_count_mod != 0 {
            post_announce_pipeline.cmd("HINCRBY").arg(&parsed.info_hash).arg("leechers").arg(leech_count_mod).ignore();
        }

        // TODO: Patch cached reply with the count mods?
        // Also invalidate existing cache
        post_announce_pipeline.cmd("DEL").arg(&cache_key).ignore();
    } else {
        post_announce_pipeline.cmd("INCR").arg(constants::NOCHANGE_ANNOUNCE_COUNT_KEY).ignore();
        // TBD: If we had a cache hit, any point to set it again? 
        // For now we are ok, since background pipeline, O(1) in redis.
        post_announce_pipeline.cmd("SET").arg(&cache_key).arg(&final_res).arg("EX").arg(60 * 30).ignore();
    }


    let time_end = SystemTime::now().duration_since(UNIX_EPOCH).expect("fucked up");
    let time_end_ms: i64 = i64::try_from(time_end.as_millis()).expect("fucc");

    let req_duration = time_end_ms - time_now_ms;

    post_announce_pipeline.cmd("INCR").arg(constants::ANNOUNCE_COUNT_KEY).ignore();
    post_announce_pipeline.cmd("INCRBY").arg(constants::REQ_DURATION_KEY).arg(req_duration).ignore();


    actix_web::rt::spawn(async move {
        // log the summary
        // TODO: For now removed this since we no longer have string IP
        // in future can enable via compilation feature
        // post_announce_pipeline.cmd("PUBLISH").arg("reqlog").arg(req_log::generate_csv(&user_ip_owned, &parsed.info_hash)).ignore();


        let () = match post_announce_pipeline.query_async::<redis::aio::MultiplexedConnection, ()>(&mut rc).await {
            Ok(_) => (),
            Err(e) => {
                println!("Err during pipe {}. Timenow: {}, scountmod: {}, lcountmod: {}", e, time_now_ms, seed_count_mod, leech_count_mod);
                ()
            },
        };
    });

    return HttpResponse::build(StatusCode::OK).append_header(header::ContentType::plaintext()).body(final_res);
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

struct AppState {
    redis_connection: redis::aio::MultiplexedConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let redis_host = args.redis_host.unwrap_or_else(|| "127.0.0.1:6379".to_string());
    let redis = redis::Client::open("redis://".to_string() + &redis_host).unwrap();
    let redis_connection = redis.get_multiplexed_tokio_connection().await.unwrap();

    let data = web::Data::new(AppState{
        redis_connection,
    });

    let port = args.port.unwrap_or_else(|| 6969);
    let host = args.host.unwrap_or_else(|| "0.0.0.0".to_string());

    return HttpServer::new(move || {
        App::new()
        .app_data(data.clone())
        .service(healthz)
        .service(announce)
    })
    .bind((host, port))?
    .max_connection_rate(8192)
    .keep_alive(None)
    .run()
    .await;
}
