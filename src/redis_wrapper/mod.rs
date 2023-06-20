use opentelemetry::{global, trace::Tracer};
use redis;

pub async fn healthcheck(c: &mut redis::aio::MultiplexedConnection) -> bool {
    let tracer = global::tracer("healthcheck");
    tracer.in_span("healthcheck", |ctx| async move {
        match redis::cmd("PING").query_async::<redis::aio::MultiplexedConnection, ()>(c).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }).await
}