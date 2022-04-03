mod tests;

use hex;

pub fn do_nothing() {
    println!("XD");
}

pub fn url_encoded_to_hex(urlenc: &str) -> String {
    let mut hex_str = String::new();
    let mut i = 0;

    while i < urlenc.chars().count() {
        let the_char = urlenc.chars().nth(i).expect("Shits fucked up yo");       

        if the_char == '%' {
            hex_str.push_str(&urlenc[i+1..i+3]);
            i += 3;
        } else {
            hex_str.push_str(&hex::encode(the_char.to_string()));
            i += 1;
        }
    }

    return hex_str.to_lowercase();
}

// Nooby way to convery an IPv4 string and a u16 port into vector of bytes
// This gives us the "tuple" of (ip, port) of the torrent client as 6 bytes
// Which we will store directly into redis as is
// Error Handling: 0 (for now...)
pub fn ip_str_port_u16_to_bytes_u8(ip_str: &str, port: u16) -> [u8; 6] {
    let mut result: [u8; 6] = [0; 6];
    let mut parts = ip_str.split('.');

    for i in 0..4 {
        result[i] = match parts.next() {
            Some(v) => match v.parse::<u8>() {
                Ok(v) => v,
                Err(_) => 0,
            }
            None => 0,
        }
    }    
    
    let portu8 = port.to_be_bytes();
    result[4] = portu8[0];
    result[5] = portu8[1];

    return result;
}

pub fn ip_str_port_u16_to_bytes(ip_str: &str, port: u16) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![0; 4];
    let parts: Vec<&str> = ip_str.split('.').collect();
    
    if parts.len() != 4 {
        panic!("Not 4 parts..."); // TODO: Error handling
    }

    for i in 0..4 {
        bytes[i] = parts.get(i).expect("Did not get part").parse().expect("Cannot parse into u8");
    }

    // println!("Bytes is now {:?}", bytes);
    
    bytes.append(&mut Vec::from(port.to_be_bytes()));

    // println!("Bytes is now {:?}", bytes);
    return bytes;
}
