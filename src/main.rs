#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod byte_functions;
mod query;
mod constants;
mod req_log;
mod db;
mod handlers;
mod blacklist;
mod store;

use actix_web::{dev::Service, get, http::{header, StatusCode}, web::{self, Redirect}, App, HttpRequest, HttpResponse, HttpServer};
use handlers::healthz::healthz;
use handlers::metrics::metrics;
use handlers::test_seed::{test_peer_exists, test_seed};
use blacklist::{Blacklist, Action, load_blacklist};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use clap::Parser;
use store::{AnnounceEvent, AnnounceInput, TrackerStore, SWEEP_INTERVAL};

#[cfg(feature = "tracing")]
use opentelemetry::{global, sdk::trace as sdktrace, trace::{TraceContextExt, FutureExt, TraceError, Tracer, get_active_span}, Key, KeyValue};
#[cfg(feature = "tracing")]
use opentelemetry_otlp::WithExportConfig;
#[cfg(feature = "tracing")]
use opentelemetry::trace::Span;

mod tracing;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    port: Option<u16>,

    #[arg(long)]
    host: Option<String>,

    #[arg(long)]
    blacklist: Option<String>,

    /// Expose /test/seed and /test/peer-exists for integration tests
    #[arg(long)]
    enable_test_endpoints: bool,

    #[cfg(feature = "tracing")]
    #[arg(long)]
    otlp_endpoint: Option<String>,
}

#[get("/announce")]
async fn announce(req: HttpRequest, data: web::Data<AppState>) -> HttpResponse {
    let time_now = SystemTime::now().duration_since(UNIX_EPOCH).expect("time went backwards");
    let time_now_ms: i64 = i64::try_from(time_now.as_millis()).expect("timestamp overflow");

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

    let parsed = match query::parse_announce(user_ip, query.replace('%', "%25").as_bytes()) {
        Ok(legit) => legit,
        Err(query::QueryError::ParseFailure) => {
            trace_log!("failed to parse announce");
            return HttpResponse::build(StatusCode::BAD_REQUEST).body("Failed to parse announce\n");
        }
        Err(query::QueryError::InvalidInfohash) => {
            trace_log!("invalid infohash");
            return HttpResponse::build(StatusCode::BAD_REQUEST).body("Infohash is not 20 bytes\n");
        }
    };

    if let Some(action) = data.blacklist.lookup(&parsed.info_hash.0) {
        return match action {
            Action::Block => HttpResponse::build(StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS).finish(),
            Action::Redirect(url) => HttpResponse::TemporaryRedirect()
                .insert_header((header::LOCATION, url.as_str()))
                .finish(),
        };
    }

    let event = match parsed.event {
        query::Event::Stopped => AnnounceEvent::Stopped,
        query::Event::Completed => AnnounceEvent::Completed,
        query::Event::Unknown => AnnounceEvent::Unknown,
    };

    let time_before_store = SystemTime::now().duration_since(UNIX_EPOCH).expect("time went backwards");
    let body = data.store.handle_announce(
        AnnounceInput {
            info_hash: parsed.info_hash.0,
            peer: parsed.ip_port,
            is_seeding: parsed.is_seeding,
            event,
        },
        i64::try_from(time_before_store.as_millis()).expect("timestamp overflow") - time_now_ms,
    );

    #[cfg(feature = "tracing")]
    {
        get_active_span(|span| {
            let infohash = String::from_utf8_lossy(&parsed.info_hash.0).to_string();
            span.set_attribute(Key::new("infohash").string(infohash));
            span.add_event("finished", vec![]);
        })
    }

    HttpResponse::build(StatusCode::OK)
        .append_header(header::ContentType::plaintext())
        .body(body)
}

struct AppState {
    store: Arc<TrackerStore>,
    active_requests: Mutex<u32>,
    blacklist: Blacklist,
}

struct ActiveRequestGuard {
    data: web::Data<AppState>,
}

impl Drop for ActiveRequestGuard {
    fn drop(&mut self) {
        *self.data.active_requests.lock() -= 1;
    }
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
                opentelemetry::KeyValue::new("service.namespace", "kiryuu-namespace"),
                opentelemetry::KeyValue::new("exporter", "alloy"),
            ]),
        ))
        .install_batch(opentelemetry::runtime::Tokio)
}

static HOMEPAGE: &str = "https://tracker.mywaifu.best";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    #[cfg(feature = "tracing")]
    {
        let _tracer = init_tracer(&args).expect("Failed to initialise tracer.");
    }

    let blacklist = match &args.blacklist {
        Some(path) => load_blacklist(path).unwrap(),
        None => Blacklist::new(),
    };

    let store = Arc::new(TrackerStore::new());

    {
        let store_sweeper = store.clone();
        actix_web::rt::spawn(async move {
            let mut interval = actix_web::rt::time::interval(SWEEP_INTERVAL);
            loop {
                interval.tick().await;
                store_sweeper.sweep_stale_torrents();
            }
        });
    }

    let data = web::Data::new(AppState {
        store,
        active_requests: Mutex::new(0),
        blacklist,
    });

    let port = args.port.unwrap_or(6969);
    let host = args.host.unwrap_or_else(|| "0.0.0.0".to_string());
    let enable_test_endpoints = args.enable_test_endpoints;

    let data_metrics = data.clone();

    let metrics_server = HttpServer::new(move || {
        App::new()
            .app_data(data_metrics.clone())
            .service(metrics)
    })
    .bind(("127.0.0.1", 6868))?
    .run();

    let main_server = HttpServer::new(move || {
        let mut app = App::new()
            .app_data(data.clone())
            .wrap_fn(|req, srv| {
                #[cfg(feature = "tracing")]
                {
                    let tracer = global::tracer("http");
                    tracer.in_span(req.path().to_string(), move |cx| {
                        cx.span().set_attribute(Key::new("path").string(req.path().to_string()));
                        if let Some(val) = req.peer_addr() {
                            cx.span().set_attribute(Key::new("ip").string(val.ip().to_string()));
                        }
                        match req.headers().get(header::USER_AGENT) {
                            Some(val) => cx.span().set_attribute(Key::new("user-agent").string(val.to_str().unwrap_or("ERR").to_owned())),
                            None => cx.span().set_attribute(Key::new("user-agent").string("NA"))
                        };
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
                    let data = req.app_data::<web::Data<AppState>>().unwrap().clone();
                    *data.active_requests.lock() += 1;
                    let guard = ActiveRequestGuard { data: data.clone() };
                    let fut = srv.call(req);
                    async move {
                        let _guard = guard;
                        fut.await
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
            }));

        if enable_test_endpoints {
            app = app.service(test_seed).service(test_peer_exists);
        }

        app
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
    .run();

    actix_web::rt::spawn(metrics_server);
    main_server.await
}
