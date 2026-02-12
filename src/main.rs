mod byte_functions;
mod query;
mod constants;
mod req_log;
mod db;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, dev::Service, error::ErrorNotFound, get, http::{StatusCode, header}, middleware, rt::time::sleep, web::{self, Redirect}};
use db::get_hash_keys_scan;
use std::{ops::{Add, AddAssign, Sub, SubAssign}, sync::Mutex, time::{Duration, SystemTime, UNIX_EPOCH}};
use clap::Parser;
use std::collections::HashMap;

#[cfg(feature = "tracing")]
use opentelemetry::{global, sdk::trace as sdktrace, trace::{TraceContextExt, FutureExt, TraceError, Tracer, get_active_span}, Key, KeyValue};
#[cfg(feature = "tracing")]
use opentelemetry_otlp::WithExportConfig;
#[cfg(feature = "tracing")]
use opentelemetry::trace::Span;

// This will acutally always be imported, has the feature flag
// inside the macro.
mod tracing;
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

    /// Redis connection string
    /// For TCP: redis://127.0.0.1:6379
    /// For Unix: unix:///tmp/redis.sock
    #[arg(long)]
    redis_connection_string: Option<String>,
    
    /// Address of redis instance. Default: 127.0.0.1:6379
    #[arg(long)]
    redis_host: Option<String>,

    #[cfg(feature = "tracing")]
    /// Address of OTLP consumer. Default: http://127.0.0.1:4317 (Grafana Alloy)
    #[arg(long)]
    otlp_endpoint: Option<String>,
}

// If not more than 31, possible not online
// So dont waste bandwidth on redis query etc.
const THIRTY_ONE_MINUTES_AS_SECONDS: i64 = 60 * 31;

#[derive(Debug)]
enum Exists {
    Yes,
    No,
}

impl From<&Exists> for bool {
    fn from(item: &Exists) -> Self {
        match item {
            Exists::Yes => true,
            Exists::No => false,
        }
    }
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
                trace_log!("failed to parse announce");
                return HttpResponse::build(StatusCode::BAD_REQUEST).body("Failed to parse announce\n");
            },
            query::QueryError::InvalidInfohash => {
                trace_log!("invalid infohash");
                return HttpResponse::build(StatusCode::BAD_REQUEST).body("Infohash is not 20 bytes\n");
            }
        }
    };

    // Get seeders & leechers
    let mut rc = data.redis_connection.clone();
    let (seeders_key, leechers_key, cache_key) = byte_functions::make_redis_keys(&parsed.info_hash);

    let mut p = redis::pipe();
    let pp = p.hexists(&seeders_key, &parsed.ip_port)
    .hexists(&leechers_key, &parsed.ip_port)
    .get(&cache_key);
    
    let (is_seeder_v2, is_leecher_v2, cached_reply) : (bool, bool, Vec<u8>) = trace_wrap_v2!(pp.query_async(&mut rc).await, "redis", "seeder_leecher_cache").unwrap();

    let mut post_announce_pipeline = redis::pipe();

    // update / reset the expire time on the hashes
    post_announce_pipeline.expire(&seeders_key, THIRTY_ONE_MINUTES_AS_SECONDS).ignore()
    .expire(&leechers_key, THIRTY_ONE_MINUTES_AS_SECONDS).ignore();

    // These will contain how we change the total number of seeders / leechers by the end of the announce
    let mut seed_count_mod: i64 = 0;
    let mut leech_count_mod: i64 = 0;


    if let query::Event::Stopped = parsed.event {
        if is_seeder_v2 {
            seed_count_mod -= 1;
            post_announce_pipeline.hdel(&seeders_key, &parsed.ip_port).ignore();
        } else if is_leecher_v2 {
            leech_count_mod -= 1;
            post_announce_pipeline.hdel(&leechers_key, &parsed.ip_port).ignore();
        }
    } else if parsed.is_seeding {
        // New seeder, we will HSET it
        if is_seeder_v2 == false {
            seed_count_mod += 1;
            post_announce_pipeline.hset(&seeders_key, &parsed.ip_port, 1).ignore();
        }

        // regardless of new or not, we will always set HEXPIRE to 31 min from now
        post_announce_pipeline.hexpire(&seeders_key, 60 * 31, redis::ExpireOption::NONE, &parsed.ip_port).ignore();

        // They just completed
        if let query::Event::Completed = parsed.event {
            // If they were previously leecher, remove from that pool
            if is_leecher_v2 {
                post_announce_pipeline.hdel(&leechers_key, &parsed.ip_port).ignore();
                leech_count_mod -= 1
            }

            // Increment the downloaded count for the infohash stats
            // post_announce_pipeline.cmd("HINCRBY").arg(&parsed.info_hash).arg("downloaded").arg(1u32).ignore();
        }
    } else {
        // New leecher, we will HSET it
        if is_leecher_v2 == false {
            leech_count_mod += 1;
            post_announce_pipeline.hset(&leechers_key, &parsed.ip_port, 1).ignore();
        };

        // regardless of new or not, we will always set HEXPIRE to 31 min from now
        post_announce_pipeline.hexpire(&leechers_key, 60 * 31, redis::ExpireOption::NONE, &parsed.ip_port).ignore();
    } 

    // Cache miss = query redis
    // no change = update cache
    // change = clear cache

    let final_res = match cached_reply.len() {
        0 => {
            // Cache miss. Lookup from redis
            trace_log!("cache miss");
            let mut p = redis::pipe();
            // TODO: this will become 2x HKEYS w/o limit (O(log(N) + M) -> O(N)) => This could potentially be a bit of a regression; e.g. we would now get all 1000 IPs from redis (previously limit 50).
            // Alternatively: We can HSCAN till 50, which should be very efficient (Basically HSCAN is O(1) per call... , but multiple calls so latency is more? - Need to check).
            // let pp = p.hkeys(&seeders_key).hkeys(&leechers_key);
            // let (seeders, leechers) : (Vec<Vec<u8>>, Vec<Vec<u8>>) = trace_wrap_v2!(pp.query_async(&mut rc).await, "redis", "seeders_leechers").unwrap();
            let seeders = get_hash_keys_scan(&mut rc, &seeders_key, 50).await;
            let leechers = get_hash_keys_scan(&mut rc, &leechers_key, 50).await;
        
            // endex = end index XD. seems in rust cannot select first 50 elements, or limit to less if vector doesnt have 50
            // e.g. &seeders[0..50] is panicking when seeders len is < 50. Oh well.
            let seeder_endex = std::cmp::min(seeders.len(), 50);
            let leecher_endex = std::cmp::min(leechers.len(), 50);

            query::announce_reply(seeders.len() as i64 + seed_count_mod, leechers.len() as i64 + leech_count_mod, &seeders[0..seeder_endex], &leechers[0..leecher_endex])
        },
        _ => {
            trace_log!("cache hit");
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
            // post_announce_pipeline.cmd("HINCRBY").arg(&parsed.info_hash).arg("seeders").arg(seed_count_mod).ignore();
        }

        if leech_count_mod != 0 {
            // post_announce_pipeline.cmd("HINCRBY").arg(&parsed.info_hash).arg("leechers").arg(leech_count_mod).ignore();
        }

        // TODO: Patch cached reply with the count mods?
        // Also invalidate existing cache
        trace_log!("need to invalidate cache");
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


        let () = match post_announce_pipeline.query_async::<()>(&mut rc).await {
            Ok(_) => (),
            Err(e) => {
                println!("Err during pipe {}. Timenow: {}, scountmod: {}, lcountmod: {}", e, time_now_ms, seed_count_mod, leech_count_mod);
                ()
            },
        };
    });

    #[cfg(feature = "tracing")]
    {
        get_active_span(|span| {
            let infohash = String::from_utf8_lossy(&parsed.info_hash.0).to_string();
            span.set_attribute(Key::new("infohash").string(infohash));
            span.add_event("finished", vec![]);
        })
    }

    return HttpResponse::build(StatusCode::OK).append_header(header::ContentType::plaintext()).body(final_res);
}

#[get("/healthz")]
async fn healthz(data: web::Data<AppState>) -> HttpResponse {
    let mut rc = data.redis_connection.clone();
    match trace_wrap_v2!(redis::cmd("PING").query_async::<()>(&mut rc).await, "redis", "healthcheck") {
        Ok(_) => HttpResponse::build(StatusCode::OK).append_header(header::ContentType::plaintext()).body("OK\nactive_requests=".to_string() + &data.active_requests.lock().unwrap().to_string()),
        Err(_) => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).append_header(header::ContentType::plaintext()).body("OOF"),
    }
}

struct AppState {
    redis_connection: redis::aio::MultiplexedConnection,
    active_requests: Mutex<u32>,
}


#[cfg(feature = "tracing")]
fn init_tracer(args: &Args) -> Result<sdktrace::Tracer, TraceError> {
    let otlp_endpoint = args.otlp_endpoint.clone().unwrap_or_else(|| String::from("http://127.0.0.1:4317"));
    let otlp_exporter = opentelemetry_otlp::new_exporter().tonic().with_endpoint(otlp_endpoint);

    opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(otlp_exporter)
    .with_trace_config(opentelemetry::sdk::trace::config().with_resource(
        opentelemetry::sdk::Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", "kiryuu"),
            opentelemetry::KeyValue::new("service.namespace", "kiryuu-namespace"), // TBD if this is "good practice"
            opentelemetry::KeyValue::new("exporter", "alloy"), // TBD if this is "good practice"
        ]),
    ))
    .install_batch(opentelemetry::runtime::Tokio)
}

static HOMEPAGE: &'static str = "https://tracker.mywaifu.best";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    #[cfg(feature = "tracing")]
    {
        let _tracer = init_tracer(&args).expect("Failed to initialise tracer.");
    }

    let redis_host = match args.redis_connection_string {
        Some(v) => v,
        None => format!("redis://{}", args.redis_host.unwrap_or_else(|| "127.0.0.1:6379".to_string()))
    };
    
    let redis = redis::Client::open(redis_host).unwrap();
    let redis_connection = redis.get_multiplexed_tokio_connection().await.unwrap();

    let data = web::Data::new(AppState{
        redis_connection,
        active_requests: Mutex::new(0),
    });

    let port = args.port.unwrap_or_else(|| 6969);
    let host = args.host.unwrap_or_else(|| "0.0.0.0".to_string());

    return HttpServer::new(move || {
        App::new()
        .app_data(data.clone())
        .wrap_fn(|req, srv| {
            #[cfg(feature = "tracing")]
            {
                let tracer = global::tracer("http");
                tracer.in_span(req.path().to_string(), move |cx| {
                    cx.span().set_attribute(Key::new("path").string(req.path().to_string()));
                    match req.peer_addr() {
                        Some(val) => cx.span().set_attribute(Key::new("ip").string(val.ip().to_string())),
                        None => ()
                    };
                    match req.headers().get(header::USER_AGENT) {
                        Some(val) => cx.span().set_attribute(Key::new("user-agent").string(val.to_str().unwrap_or("ERR").to_owned())),
                        None => cx.span().set_attribute(Key::new("user-agent").string("NA"))
                    }
                    cx.span().add_event("starting", vec![]);
                    let fut = srv.call(req).with_context(cx.clone());

                    async move {
                        let res = fut.await?;
                        cx.span().set_attribute(Key::new("status").i64(res.status().as_u16().into()));
                        Ok(res)
                    }
                })  
            }
            #[cfg(not(feature = "tracing"))]
            {                
                {
                    let data = req.app_data::<web::Data<AppState>>().unwrap();
                    data.active_requests.lock().unwrap().add_assign(1);
                }

                let fut = srv.call(req);
                async {
                    let res = fut.await?;

                    {
                        let data = res.request().app_data::<web::Data<AppState>>().unwrap();
                        data.active_requests.lock().unwrap().sub_assign(1);
                    }
                    return Ok(res);
                }
            }
        })
        .service(healthz)
        .service(announce)
        .service(web::resource("/scrape").to(|| async {
            HttpResponse::build(StatusCode::NOT_FOUND).finish()
        }))
        .default_service(web::to(|| async {
            Redirect::to(HOMEPAGE)
        }))
    })
    .backlog(
        std::env::var("KIRYUU_ACTIX_BACKLOG")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8192)
    )
    .max_connections(
        std::env::var("KIRYUU_ACTIX_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2500)
    )
    .keep_alive(None)
    .client_request_timeout(std::time::Duration::from_millis(1000))
    .bind((host, port))?
    .run()
    .await;
}
