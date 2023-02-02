use kiryuu::byte_functions::{self};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("IP, Port to bytes");
    // Now we can perform benchmarks with this group
    group.bench_function("Bench 1", |b| b.iter(|| byte_functions::ip_str_port_u16_to_bytes(black_box("123.23.56.210"), black_box(6969)) ));
    group.bench_function("Bench 2", |b| b.iter(|| byte_functions::ip_str_port_u16_to_bytes_u8(black_box("123.23.56.210"), black_box(6969)) ));
    group.finish();


    let mut group = c.benchmark_group("URLenc to hex");
    // Now we can perform benchmarks with this group
    group.bench_function("v1", |b| b.iter(|| byte_functions::url_encoded_to_hex(black_box("%DD%00%D2%1CuDA%AAL%B6J%1E%A7z%2CvFAR%C3")) ));
    group.bench_function("v2", |b| b.iter(|| byte_functions::url_encoded_to_hex_v2(black_box("%DD%00%D2%1CuDA%AAL%B6J%1E%A7z%2CvFAR%C3")) ));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);