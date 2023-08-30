#[macro_export]
macro_rules! trace_wrap {
    ($expr:expr, $x:literal) => {{
        #[cfg(feature = "tracing")]
        {
            let tracer = global::tracer($x);
            tracer.in_span($x, |ctx| {
                ctx.span().add_event("Calling redis", vec![]);
                $expr
            })
        }

        #[cfg(not(feature = "tracing"))]
        {
            $expr
        }
    }};
}
