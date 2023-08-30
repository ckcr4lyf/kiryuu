use redis::{self, FromRedisValue};

#[cfg(feature = "tracing")]
use opentelemetry::{global, trace::{Tracer, TraceContextExt}};


pub async fn healthcheck(c: &mut redis::aio::MultiplexedConnection) -> bool {
    #[cfg(feature = "tracing")]
    {
        let tracer = global::tracer("healthcheck");
        tracer.in_span("healthcheck", |ctx| async move {
            ctx.span().add_event("Calling redis", vec![]);
            match redis::cmd("PING").query_async::<redis::aio::MultiplexedConnection, ()>(c).await {
                Ok(_) => true,
                Err(_) => false,
            }
        }).await
    }
    #[cfg(not(feature = "tracing"))]
    {
        match redis::cmd("PING").query_async::<redis::aio::MultiplexedConnection, ()>(c).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    
}

pub async fn execute_pipeline<T: FromRedisValue>(pipeline: &redis::Pipeline, c: &mut redis::aio::MultiplexedConnection) -> redis::RedisResult<T> {
    #[cfg(feature = "tracing")]
    {
        let tracer = global::tracer("execute_pipeline");
        tracer.in_span("execute_pipeline", |ctx| async move {
            ctx.span().add_event("Calling redis", vec![]);
            pipeline.query_async(c).await
        }).await
    }
    #[cfg(not(feature = "tracing"))]
    {
        pipeline.query_async(c).await
    }

}

pub async fn xd<T: FromRedisValue>(pipeline: &redis::Pipeline, c: &mut redis::aio::MultiplexedConnection) -> redis::RedisResult<T> {
    foo::trace_wrap!(pipeline.query_async(c).await, "LOL")
}


#[macro_use]
mod foo {
    macro_rules! trace_wrap {
        ($expr:expr, $x:literal) => {
            {
                #[cfg(feature = "tracing")]
                {
                    let tracer = global::tracer($x);
                    tracer.in_span($x, |ctx| async move {
                        let result = $expr;
                        result
                    }).await
                }
    
                #[cfg(not(feature = "tracing"))]
                {
                    let result = $expr;
                    result
                }
            }
        };
    }

    pub(crate) use trace_wrap;
}
