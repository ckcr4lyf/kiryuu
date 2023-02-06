use kiryuu::byte_functions;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("URLenc to hex");
    group.bench_function("v1", |b| b.iter(|| byte_functions::url_encoded_to_hex(black_box("%DD%00%D2%1CuDA%AAL%B6J%1E%A7z%2CvFAR%C3")) ));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);