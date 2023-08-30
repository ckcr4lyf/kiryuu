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
