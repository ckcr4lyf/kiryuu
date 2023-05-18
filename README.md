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

### ulimits

Make sure you set a high ulimit for open files! By default some VPS might set this to 1024, and then `kiryuu` won't be able to handle high traffic.

If you've already started kiryuu, you can identify its PID and then set it via:

```
$ prlimit --pid PID_HERE --nofile=16384:16384
```
