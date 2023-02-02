run:
	./target/debug/kiryuu

static:
	cargo build --target=x86_64-unknown-linux-musl --release

test-announce:
	curl -v "localhost:6969/announce?info_hash=AAAAAAAAAAAAAAAAAAAB&port=3333&left=0"

# kiryuu & redis should be running
gauge:
	docker run -e KIRYUU_HOST=http://172.17.0.1:6969 -e REDIS_HOST=redis://172.17.0.1:6379 ghcr.io/ckcr4lyf/kiryuu-gauge:master