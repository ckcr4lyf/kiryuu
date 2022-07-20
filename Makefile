run:
	./target/debug/kiryuu

static:
	cargo build --target=x86_64-unknown-linux-musl --release

test-announce:
	curl -v "localhost:6969/announce?info_hash=AAAAAAAAAAAAAAAAAAAB&port=3333&left=0"