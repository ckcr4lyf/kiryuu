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

#[macro_export]
macro_rules! trace_wrap_v2 {
    ($expr:expr, $x:literal) => {{
        #[cfg(feature = "tracing")]
        {
            let tracer = global::tracer("IRRELEVANT");
            let mut span = tracer.start($x);
            span.set_attribute(Key::new("bruv").string("va"));
            let x = $expr;
            span.end();
            x
        }
        #[cfg(not(feature = "tracing"))]
        {
            $expr
        }
    }};
}
