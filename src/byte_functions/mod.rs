use crate::constants;

pub mod types;

pub fn make_seeder_key(info_hash: &types::RawVal<40>) -> (types::RawVal<48>, types::RawVal<49>, types::RawVal<46>) {
    let mut seeder_key: [u8; 48] = [0; 48];
    let mut leecher_key: [u8; 49] = [0; 49];
    let mut cache_key: [u8; 46] = [0; 46];

    for i in 0..40 {
        seeder_key[i] = info_hash[i];
        leecher_key[i] = info_hash[i];
        cache_key[i] = info_hash[i]
    }

    for i in 0..8 {
        seeder_key[i] = constants::SEEDER_SUFFIX[i];
    }

    for i in 0..9 {
        leecher_key[i] = constants::LEECHER_SUFFIX[i];
    }

    for i in 0..6 {
        leecher_key[i] = constants::CACHE_SUFFIX[i];
    }

    return (types::RawVal(seeder_key), types::RawVal(leecher_key), types::RawVal(cache_key));
}

pub fn url_encoded_to_hex_u8(urlenc: &str) -> [u8; 40] {
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

    return hex_str_bytes;
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

// Convert the ipv4 addr, port combo to a [u8; 6]
pub fn ip_str_port_u16_to_bytes(ip_addr: &std::net::Ipv4Addr, port: u16) -> [u8; 6] {
    let mut result: [u8; 6] = [0; 6];
    let ip_octets = ip_addr.octets();

    for i in 0..4 {
        result[i] = ip_octets[i];
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
        // All the extra bytes will be 0x41 aka b"A"
        // "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        assert_eq!(*b"41AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", url_encoded_to_hex_u8("A"));
        assert_eq!(*b"41AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", url_encoded_to_hex_u8("%41"));
        assert_eq!(*b"4141AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", url_encoded_to_hex_u8("A%41"));
        assert_eq!(*b"4141AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", url_encoded_to_hex_u8("%41A"));
        assert_eq!(*b"4142AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", url_encoded_to_hex_u8("%41B"));
        assert_eq!(*b"4241AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", url_encoded_to_hex_u8("B%41"));
        assert_eq!(*b"4241AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", url_encoded_to_hex_u8("BA"));
        assert_eq!(*b"4142AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", url_encoded_to_hex_u8("%41%42"));

        // Add some test to make sure the hex is lowercase
        assert_eq!(*b"4d4eAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", url_encoded_to_hex_u8("MN"));
        assert_eq!(*b"1c2fAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", url_encoded_to_hex_u8("%1C%2F"));
        assert_eq!(*b"41611c2f4d4eAAAAAAAAAAAAAAAAAAAAAAAAAAAA", url_encoded_to_hex_u8("Aa%1C%2FMN"));
    }

    #[test]
    fn can_parse_ip_port() {
        assert_eq!(
            vec![127, 0, 0, 1, 13, 5],
            ip_str_port_u16_to_bytes(&std::net::Ipv4Addr::new(127, 0, 0, 1), 3333)
        );
        assert_eq!(
            vec![1, 1, 1, 1, 255, 255],
            ip_str_port_u16_to_bytes(&std::net::Ipv4Addr::new(1,1,1,1), 65535)
        ); // Shilling Cloudflare
        assert_eq!(
            vec![192, 168, 1, 1, 105, 137],
            ip_str_port_u16_to_bytes(&std::net::Ipv4Addr::new(192, 168, 1, 1), 27017)
        );
    }
}
