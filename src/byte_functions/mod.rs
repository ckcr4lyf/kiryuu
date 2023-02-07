pub fn url_encoded_to_hex(urlenc: &str) -> String {
    // Start with 40 mutable bytes on the stack
    // This allows us to write the expected hex ascii directly
    // into this array
    let mut hex_str_bytes: [u8; 40] = [0x41; 40];

    let mut pos_urlenc = 0;
    let mut pos_hex_str = 0;
    let raw = urlenc.as_bytes();
    let max = raw.len();

    while pos_urlenc < max {

        // Current character in info_hash query param
        match raw[pos_urlenc] {
            // % , meaning the next two char can be used as is (percent encoded byte)
            // Since we also want to lowercase it, we just need to set the 6th bit
            // This is because the ASCII set is designed in such a way
            // For digits (0-9), the 6th bit is already set, so its a noop
            // "A" -> 0x41 -> 0b01000001
            // "a" -> 0x61 -> 0b01100001
            // We also clear the 8th bit to guarantee it's ascii
            // allowing us to use unsafe from_utf8_unchecked()
            0x25 => {
                hex_str_bytes[pos_hex_str] = (raw[pos_urlenc+1] | 0b0010_0000) & 0b0111_1111;
                hex_str_bytes[pos_hex_str+1] = (raw[pos_urlenc+2] | 0b0010_0000) & 0b0111_1111;
                pos_hex_str += 2;
                pos_urlenc += 3;
            },
            non_pc => {
                // This byte is the actual info hash byte. So we
                // Split it into two nibbles and get their ascii
                // e.g. "A" => 0x41 => 0x4, 0x1 => "4", "1" => [0x34, 0x31]
                // Get high 4 bits (i,e 0x4_)
                let left_nibble = (0b11110000 & non_pc) >> 4;

                // Get low 4 bits (i,e 0x_1)
                let right_nibble = 0b00001111 & non_pc;

                // Get nibbles' hex characters
                hex_str_bytes[pos_hex_str] = nibble_to_ascii(left_nibble);
                hex_str_bytes[pos_hex_str+1] = nibble_to_ascii(right_nibble);
                pos_hex_str += 2;
                pos_urlenc += 1;
            }
        }
    }

    // SAFETY: The above function guarantees each u8 is within the ASCII set
    let str_val = unsafe {
        std::str::from_utf8_unchecked(&hex_str_bytes[0..pos_hex_str])
    };
    
    return str_val.to_owned();
}

// Based on some PoC, seems fastest way to convert
// A nibble to it's ascii
// https://godbolt.org/z/bcr46c7ha
#[inline(always)]
fn nibble_to_ascii(nibble: u8) -> u8 {
    if nibble < 0xA {
        nibble + 0x30
    } else {
        nibble + 0x57
    }
}


// Nooby way to convery an IPv4 string and a u16 port into a [u8; 6]
// This gives us the "tuple" of (ip, port) of the torrent client as 6 bytes
// Which we will store directly into redis as is
pub fn ip_str_port_u16_to_bytes(ip_str: &str, port: u16) -> [u8; 6] {
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

        // Add some test to make sure the hex is lowercase
        assert_eq!("4d4e", url_encoded_to_hex("MN"));
        assert_eq!("1c2f", url_encoded_to_hex("%1C%2F"));
        assert_eq!("41611c2f4d4e", url_encoded_to_hex("Aa%1C%2FMN"));

        // Add some test to make sure it can handle invalid UTF-8
        // based on the octets after the % not representing valid UTF-8
        // We can get rid of this if we move from String -> [u8; 40].
        unsafe {
            // Hacky way to ensure it is valid utf8
            let xd = url_encoded_to_hex(std::str::from_utf8_unchecked(&[0x25, 0xc3, 0x28]));
            std::str::from_utf8(xd.as_bytes()).expect("INVALID UTF-8 DETECTED!");
        }
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
