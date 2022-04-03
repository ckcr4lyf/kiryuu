#[cfg(test)]
use crate::byte_functions;

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

