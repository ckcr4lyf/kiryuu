#[macro_export]
macro_rules! trace_wrap {
    ($expr:expr, $x:literal) => {{
        #[cfg(feature = "tracing")]
        {
            let tracer = global::tracer($x);
            tracer
                .in_span($x, |ctx| async move {
                    let result = $expr;
                    result
                })
                .await
        }

        #[cfg(not(feature = "tracing"))]
        {
            let result = $expr;
            result
        }
    }};
}
