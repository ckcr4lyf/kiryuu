use kiryuu::byte_functions::{self};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("URLenc to hex");
    group.bench_function("v1", |b| b.iter(|| byte_functions::url_encoded_to_hex(black_box("%DD%00%D2%1CuDA%AAL%B6J%1E%A7z%2CvFAR%C3")) ));
    group.bench_function("v2", |b| b.iter(|| byte_functions::url_encoded_to_hex_v2(black_box("%DD%00%D2%1CuDA%AAL%B6J%1E%A7z%2CvFAR%C3")) ));
    group.bench_function("v3", |b| b.iter(|| byte_functions::xd(black_box("%DD%00%D2%1CuDA%AAL%B6J%1E%A7z%2CvFAR%C3")) ));
    group.bench_function("v4", |b| b.iter(|| byte_functions::xd2(black_box("%DD%00%D2%1CuDA%AAL%B6J%1E%A7z%2CvFAR%C3")) ));
    group.bench_function("v5", |b| b.iter(|| byte_functions::xd3(black_box("%DD%00%D2%1CuDA%AAL%B6J%1E%A7z%2CvFAR%C3")) ));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);