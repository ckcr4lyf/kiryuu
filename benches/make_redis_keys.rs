use kiryuu::byte_functions;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Make redis keys");

    let ih = "41AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string();
    group.bench_function("u8", |b| b.iter(|| byte_functions::make_redis_keys(black_box(&byte_functions::types::RawVal(*b"41AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);