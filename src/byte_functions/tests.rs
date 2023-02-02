#[cfg(test)]
use crate::byte_functions;

// use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[test]
fn is_legit(){
    assert_eq!("41", byte_functions::url_encoded_to_hex("A"));
    assert_eq!("41", byte_functions::url_encoded_to_hex("%41"));
    assert_eq!("4141", byte_functions::url_encoded_to_hex("A%41"));
    assert_eq!("4141", byte_functions::url_encoded_to_hex("%41A"));
    assert_eq!("4142", byte_functions::url_encoded_to_hex("%41B"));
    assert_eq!("4241", byte_functions::url_encoded_to_hex("B%41"));
    assert_eq!("4241", byte_functions::url_encoded_to_hex("BA"));
    assert_eq!("4142", byte_functions::url_encoded_to_hex("%41%42"));
}

#[test]
fn is_legit_v2(){
    assert_eq!("41", byte_functions::url_encoded_to_hex_v2("A"));
    assert_eq!("41", byte_functions::url_encoded_to_hex_v2("%41"));
    assert_eq!("4141", byte_functions::url_encoded_to_hex_v2("A%41"));
    assert_eq!("4141", byte_functions::url_encoded_to_hex_v2("%41A"));
    assert_eq!("4142", byte_functions::url_encoded_to_hex_v2("%41B"));
    assert_eq!("4241", byte_functions::url_encoded_to_hex_v2("B%41"));
    assert_eq!("4241", byte_functions::url_encoded_to_hex_v2("BA"));
    assert_eq!("4142", byte_functions::url_encoded_to_hex_v2("%41%42"));
}

#[test]
fn is_legit_xd(){
    assert_eq!("41", byte_functions::xd("A"));
    assert_eq!("41", byte_functions::xd("%41"));
    assert_eq!("4141", byte_functions::xd("A%41"));
    assert_eq!("4141", byte_functions::xd("%41A"));
    assert_eq!("4142", byte_functions::xd("%41B"));
    assert_eq!("4241", byte_functions::xd("B%41"));
    assert_eq!("4241", byte_functions::xd("BA"));
    assert_eq!("4142", byte_functions::xd("%41%42"));
}

#[test]
fn is_legit_v4(){
    // assert_eq!("41", byte_functions::url_encoded_to_hex_v4("A"));
    // assert_eq!("41", byte_functions::url_encoded_to_hex_v4("%41"));
    // assert_eq!("4141", byte_functions::url_encoded_to_hex_v4("A%41"));
    // assert_eq!("4141", byte_functions::url_encoded_to_hex_v4("%41A"));
    // assert_eq!("4142", byte_functions::url_encoded_to_hex_v4("%41B"));
    // assert_eq!("4241", byte_functions::url_encoded_to_hex_v4("B%41"));
    // assert_eq!("4241", byte_functions::url_encoded_to_hex_v4("BA"));
    // assert_eq!("4142", byte_functions::url_encoded_to_hex_v4("%41%42"));
}

#[test]
fn can_parse_ip_port(){
    assert_eq!(vec![127, 0, 0, 1, 13, 5], byte_functions::ip_str_port_u16_to_bytes("127.0.0.1", 3333));
    assert_eq!(vec![1, 1, 1, 1, 255, 255], byte_functions::ip_str_port_u16_to_bytes("1.1.1.1", 65535)); // Shilling Cloudflare
    assert_eq!(vec![192, 168, 1, 1, 105, 137], byte_functions::ip_str_port_u16_to_bytes("192.168.1.1", 27017));
}


#[test]
fn can_parse_ip_port_u8(){
    assert_eq!([127, 0, 0, 1, 13, 5], byte_functions::ip_str_port_u16_to_bytes_u8("127.0.0.1", 3333));
    assert_eq!([1, 1, 1, 1, 255, 255], byte_functions::ip_str_port_u16_to_bytes_u8("1.1.1.1", 65535)); // Shilling Cloudflare
    assert_eq!([192, 168, 1, 1, 105, 137], byte_functions::ip_str_port_u16_to_bytes_u8("192.168.1.1", 27017));
}

#[test]
fn compare(){
    for _ in 0..10000000 {
        byte_functions::ip_str_port_u16_to_bytes("127.0.0.1", 3333);
        byte_functions::ip_str_port_u16_to_bytes_u8("127.0.0.1", 3333);
    }
}

// %DD%00%D2%1CuDA%AAL%B6J%1E%A7z%2Cv%5D%E2%1F%AC

#[test]
fn compareUrlToHex(){
    for _ in 0..100000000 {
        byte_functions::url_encoded_to_hex_v3("%DD%00%D2%1CuDA%AAL%B6J%1E%A7z%2CvFAR%C3");
        byte_functions::url_encoded_to_hex_v5("%DD%00%D2%1CuDA%AAL%B6J%1E%A7z%2CvFAR%C3");
    }
}


// fn criterion_benchmark(c: &mut Criterion) {
//     c.bench_function("fib 20", |b| b.iter(|| byte_functions::ip_str_port_u16_to_bytes(black_box("127.0.0.1"), black_box(3333))));
// }

// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);