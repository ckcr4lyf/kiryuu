use opentelemetry::{global, trace::Tracer};
use redis::{self, FromRedisValue};

pub async fn healthcheck(c: &mut redis::aio::MultiplexedConnection) -> bool {
    let tracer = global::tracer("healthcheck");
    tracer.in_span("healthcheck", |ctx| async move {
        match redis::cmd("PING").query_async::<redis::aio::MultiplexedConnection, ()>(c).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }).await
}

pub async fn execute_pipeline<T: FromRedisValue>(pipeline: &redis::Pipeline, c: &mut redis::aio::MultiplexedConnection) -> redis::RedisResult<T> {
    let tracer = global::tracer("execute_pipeline");
    tracer.in_span("execute_pipeline", |ctx| async move {
        pipeline.query_async(c).await
    }).await
}
