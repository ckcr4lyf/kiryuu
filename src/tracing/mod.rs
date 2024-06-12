#[macro_export]
macro_rules! trace_wrap_v2 {
    ($expr:expr, $loggername:literal, $commandname:literal) => {{
        #[cfg(feature = "tracing")]
        {
            let tracer = global::tracer($loggername);
            let mut span = tracer.start($commandname);
            // span.set_attribute(Key::new("bruv").string("va"));
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

#[macro_export]
macro_rules! trace_log {
    ($logline:literal) => {{
        #[cfg(feature = "tracing")]
        {
            get_active_span(|span| {
                span.add_event($logline, vec![]);
            })
        }
        #[cfg(not(feature = "tracing"))]
        {
            // noop
        }
    }};
}
