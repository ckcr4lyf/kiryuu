# kiryuu

Rewrite of [kouko](https://github.com/ckcr4lyf/kouko) in Rust, for better performance!

Kiryuu powers `http://tracker.mywaifu.best:6969/announce`

## Thanks

Many thanks to horsie and anon from Discord, both of whom were extremely helpful in helping me get more familiar with rust, and for offering their heads as I bounced ideas across them.

## Usage

The current release can be considered stable, but is not intended for use by others - it is not very customizable yet. That said, feel free to hack around with it if you like!

### Building

Best to build in release mode and target your CPU natively for better performance.

```
$ RUSTFLAGS="-C target-cpu=native" cargo build --release
```

Building a static binary is possible with the `musl` target, with something like:

```
$ cargo build --target=x86_64-unknown-linux-musl --release

```

### ulimits

Make sure you set a high ulimit for open files! By default some VPS might set this to 1024, and then `kiryuu` won't be able to handle high traffic.

If you've already started kiryuu, you can identify its PID and then set it via:

```
$ prlimit --pid PID_HERE --nofile=16384:16384
```

## Testing

There are integration tests via Gauge that run in CI. The tests are located at https://github.com/ckcr4lyf/kiryuu-gauge

To run them locally, you could use:

```
$ docker run -e KIRYUU_HOST=http://172.17.0.1:6969 -e REDIS_HOST=redis://172.17.0.1:6379 ghcr.io/ckcr4lyf/kiryuu-gauge:master
```

(Make sure you've kiryuu running locally and redis as well!)

### Dummy cURL

Or you can just send an example cURL 

```
curl "localhost:6969/announce?info_hash=AAAAAAAAAAAAAAAAAAAA&port=1337&left=0" 
```

## Tracing

To build with tracing, enable the tracing feature:

```
$ RUSTFLAGS="-C target-cpu=native" cargo build --release --features tracing
```

Kiryuu currently supports exporting traces via an OTLP endpoint. E.g. you can run a collector via [The OTEL quick start](https://opentelemetry.io/docs/collector/quick-start/).

Or use [Grafana Cloud](https://grafana.com/products/cloud/) w/ [Grafana Alloy](https://grafana.com/docs/alloy/latest/).
