use hex;

pub fn url_encoded_to_hex(urlenc: &str) -> String {
    let mut hex_str = String::new();
    let mut i = 0;

    while i < urlenc.chars().count() {
        let the_char = urlenc.chars().nth(i).expect("Shits fucked up yo");

        if the_char == '%' {
            hex_str.push_str(&urlenc[i + 1..i + 3]);
            i += 3;
        } else {
            hex_str.push_str(&hex::encode(the_char.to_string()));
            i += 1;
        }
    }

    return hex_str.to_lowercase();
}


/// HEX_CHAR_MAP is an index from a nibble to its ASCII representation
/// Since nibble values are one of 16 hex digits (0-F), this is a 16
/// character array that maps to the corresponding hex character
/// e.g. , the nib 0xA -> 0x41 -> "A" (TODO: fix to lowercase)
const HEX_CHAR_MAP: [u8; 16] = [0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46];



// Nooby way to convery an IPv4 string and a u16 port into vector of bytes
// This gives us the "tuple" of (ip, port) of the torrent client as 6 bytes
// Which we will store directly into redis as is
// Error Handling: 0 (for now...)
pub fn ip_str_port_u16_to_bytes(ip_str: &str, port: u16) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![0; 4];
    let parts: Vec<&str> = ip_str.split('.').collect();

    if parts.len() != 4 {
        panic!("Not 4 parts..."); // TODO: Error handling
    }

    for i in 0..4 {
        bytes[i] = parts
            .get(i)
            .expect("Did not get part")
            .parse()
            .expect("Cannot parse into u8");
    }

    // println!("Bytes is now {:?}", bytes);

    bytes.append(&mut Vec::from(port.to_be_bytes()));

    // println!("Bytes is now {:?}", bytes);
    return bytes;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_legit() {
        assert_eq!("41", url_encoded_to_hex("A"));
        assert_eq!("41", url_encoded_to_hex("%41"));
        assert_eq!("4141", url_encoded_to_hex("A%41"));
        assert_eq!("4141", url_encoded_to_hex("%41A"));
        assert_eq!("4142", url_encoded_to_hex("%41B"));
        assert_eq!("4241", url_encoded_to_hex("B%41"));
        assert_eq!("4241", url_encoded_to_hex("BA"));
        assert_eq!("4142", url_encoded_to_hex("%41%42"));
    }

    #[test]
    fn can_parse_ip_port() {
        assert_eq!(
            vec![127, 0, 0, 1, 13, 5],
            ip_str_port_u16_to_bytes("127.0.0.1", 3333)
        );
        assert_eq!(
            vec![1, 1, 1, 1, 255, 255],
            ip_str_port_u16_to_bytes("1.1.1.1", 65535)
        ); // Shilling Cloudflare
        assert_eq!(
            vec![192, 168, 1, 1, 105, 137],
            ip_str_port_u16_to_bytes("192.168.1.1", 27017)
        );
    }
}
