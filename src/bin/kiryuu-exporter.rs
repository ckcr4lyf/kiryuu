use clap::Parser;
use actix_web::{self, get, http::{header, StatusCode}, web, App, HttpResponse, HttpServer};
use kiryuu::constants;

#[derive(Debug, Parser)]
struct Args {
    // Address of redis instance. Default: 127.0.0.1:6379
    #[arg(long)]
    redis_host: Option<String>
}

struct AppState {
    redis_connection: redis::aio::MultiplexedConnection,
    postgres_client: tokio_postgres::Client,
}

#[get("/metrics")]
async fn metrics(data: web::Data<AppState>) -> HttpResponse {
    let mut redis_connection = data.redis_connection.clone();
    
    let (announce_count, req_duration, cache_hit_count, nochange_count): (i64, i64, i64, i64) = redis::cmd("MGET").arg(constants::ANNOUNCE_COUNT_KEY).arg(constants::REQ_DURATION_KEY).arg(constants::CACHE_HIT_ANNOUNCE_COUNT_KEY).arg(constants::NOCHANGE_ANNOUNCE_COUNT_KEY).query_async(&mut redis_connection).await.expect("fucc");

    // counting rows is expensive, takes like 2 seconds
    // let rows = data.postgres_client.query("SELECT COUNT(*) FROM torrents WHERE  last_announce > (extract(epoch from now()) * 1000) - 1000 * 60 * 31;", &[]).await.expect("fucc psql");

    // let active_count: i64 = rows[0].get(0);

    let response = format!(r#"
kiryuu_http_nochange_request_count{{status_code="200", method="GET", path="announce"}} {}
kiryuu_http_cache_hit_request_count{{status_code="200", method="GET", path="announce"}} {}
kiryuu_http_request_count{{status_code="200", method="GET", path="announce"}} {}
kiryuu_http_request_duration_sum{{status_code="200", method="GET", path="announce"}} {}
    
    "#, nochange_count, cache_hit_count, announce_count, req_duration);

    HttpResponse::build(StatusCode::OK).append_header(header::ContentType::plaintext()).body(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let args = Args::parse();
    let redis_host = args.redis_host.unwrap_or_else(|| "127.0.0.1:6379".to_string());
    let redis = redis::Client::open("redis://".to_string() + &redis_host).unwrap();
    let redis_connection = redis.get_multiplexed_tokio_connection().await.unwrap();

    let (client, connection) = tokio_postgres::connect("host=localhost user=postgres password=password", tokio_postgres::NoTls).await.unwrap();

    // Spawn off connection into its own guy
    actix_web::rt::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let data = web::Data::new(AppState{
        redis_connection,
        postgres_client: client,
    });

    return HttpServer::new(move || {
        App::new()
        .app_data(data.clone())
        .service(metrics)
    })
    .bind(("127.0.0.1", 9999))?
    .run()
    .await;
}