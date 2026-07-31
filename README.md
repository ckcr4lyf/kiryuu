# kiryuu

Rewrite of [kouko](https://github.com/ckcr4lyf/kouko) in Rust, for better performance!

Kiryuu powers `http://tracker.mywaifu.best:6969/announce`

MyWaifu runs on a Hetzner CPX11 (2vCPU, 2GB RAM) and serves ~290M requests a day (~3000-4000req/s)

![resource usage](https://github.com/user-attachments/assets/cb443b41-6333-4170-8fd7-76615786df6f)

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

### Actix tuning

There are a couple of options configurable via environment variables

| **Environment variable**            | **Description**                                | **Default** |
| ----------------------------------- | ---------------------------------------------- | ---------- |
| `KIRYUU_ACTIX_BACKLOG`             | Maximum number of pending connections in queue | `8192`     |
| `KIRYUU_ACTIX_MAX_CONNECTIONS`     | Maximum number of concurrent connections       | `2500`     |

From some testing on Hetzner, it works best when run as:

```
KIRYUU_ACTIX_BACKLOG=4096 KIRYUU_ACTIX_MAX_CONNECTIONS=500 ./kiryuu --blacklist /tmp/blacklist.txt
```

With the ulimit for open files set to `4096`. For more around tuning, [see this issue](https://github.com/ckcr4lyf/kiryuu/issues/53)

### ulimits

Make sure you set a high ulimit for open files! By default some VPS might set this to 1024, and then `kiryuu` won't be able to handle high traffic, e.g.:

```
ulimit -n 4096
```

If you've already started kiryuu, you can identify its PID and then set it via:

```
$ prlimit --pid PID_HERE --nofile=4096:4096
```

## Testing

There are integration tests via Gauge that run in CI. The tests are located at https://github.com/ckcr4lyf/kiryuu-gauge

To run them locally, you could use:

```
$ docker run -e KIRYUU_HOST=http://172.17.0.1:6969 ghcr.io/ckcr4lyf/kiryuu-gauge:master
```

The suite needs the test-only endpoints (`/test/seed` and `/test/peer-exists`), so make sure you've got kiryuu running locally with `--enable-test-endpoints`:

```
$ ./kiryuu --blacklist __fixtures__/blacklist.txt --enable-test-endpoints
```

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

## Running as a systemd service

A unit template is included at [`kiryuu.service`](./kiryuu.service). It sets the required `LimitNOFILE` (see [ulimits](#ulimits)) and the Actix tuning values from above, so you don't have to remember them per-invocation.

To install:

```
$ sudo cp target/release/kiryuu /usr/local/bin/kiryuu
$ sudo mkdir -p /etc/kiryuu && sudo cp blacklist.txt /etc/kiryuu/blacklist.txt
$ sudo cp kiryuu.service /etc/systemd/system/
$ sudo systemctl daemon-reload
$ sudo systemctl enable --now kiryuu
```

Then check on it with:

```
$ systemctl status kiryuu
$ journalctl -u kiryuu -f
```

The unit assumes the binary is at `/usr/local/bin/kiryuu` and the blacklist at `/etc/kiryuu/blacklist.txt` — edit `ExecStart` if you put them elsewhere. Note that `PrivateTmp=yes` gives the service its own `/tmp`, so a blacklist under `/tmp` won't be visible to it; either keep the blacklist outside `/tmp` or drop that line.

It runs under `DynamicUser=yes` (a transient, unprivileged user), which works because kiryuu listens on 6969 and keeps no state on disk. Swap in `User=` if you'd rather have a fixed account.

### Restarts

`Restart=always` will bring kiryuu back up if it dies, but note that **all swarm state is in memory** — a restart drops every peer until clients re-announce.

Startup fails hard if `--blacklist` points at a file that can't be read, so `StartLimitBurst=5` stops systemd from retrying forever on a missing blacklist; after 5 failures in 60s the unit stays `failed` where you'll notice it. Clear that state once you've fixed the file:

```
$ sudo systemctl reset-failed kiryuu
```
