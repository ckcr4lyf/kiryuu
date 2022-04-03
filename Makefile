run:
	./target/debug/kiryuu

static:
	cargo build --target=x86_64-unknown-linux-musl --release