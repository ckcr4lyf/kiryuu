use kiryuu::byte_functions;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("IP str to bytes");
    group.bench_function("u8", |b| b.iter(|| byte_functions::ip_str_port_u16_to_bytes(black_box("123.45.99.31"), black_box(27017)) ));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);