pub mod types;

pub fn make_redis_keys(infohash: &types::RawVal<20>) -> (types::RawVal<22>, types::RawVal<22>, types::RawVal<22>) {
    let mut seeder_key: [u8; 22] = *b"AAAAAAAAAAAAAAAAAAAA:s";
    let mut leecher_key: [u8; 22] = *b"AAAAAAAAAAAAAAAAAAAA:l";
    let mut cache_key: [u8; 22] = *b"AAAAAAAAAAAAAAAAAAAA:c";

    seeder_key[0..20].copy_from_slice(&infohash.0);
    leecher_key[0..20].copy_from_slice(&infohash.0);
    cache_key[0..20].copy_from_slice(&infohash.0);

    return (types::RawVal(seeder_key), types::RawVal(leecher_key), types::RawVal(cache_key));
}

// TODO: return Result for malformed infohashes (e.g. "%A" -> Will trigger out-of-index)
pub fn url_encoded_to_raw_u8(urlenc: &str) -> [u8; 20] {
    // Start with 20 mutable bytes on the stack
    // This allows us to write the expected hex ascii directly
    // into this array
    let mut raw_infohash: [u8; 20] = [0x00; 20];

    let mut pos_urlenc = 0;
    let mut pos_raw_infohash = 0;
    let raw_urlenc = urlenc.as_bytes();
    let max = raw_urlenc.len();

    while pos_urlenc < max {

        // Current character in info_hash query param
        match raw_urlenc[pos_urlenc] {
            // % , meaning the next two char are hex representation of raw byte
            // e.g. %FA -> The byte 0xFA
            0x25 => {
                raw_infohash[pos_raw_infohash] = (ascii_to_nibble(raw_urlenc[pos_urlenc+1]) << 4) | ascii_to_nibble(raw_urlenc[pos_urlenc+2]);
                pos_raw_infohash += 1;
                pos_urlenc += 3;
            },
            _ => {
                // This byte is the actual info hash byte. So we
                // can use it as is
                raw_infohash[pos_raw_infohash] = raw_urlenc[pos_urlenc];
                pos_raw_infohash += 1;
                pos_urlenc += 1;
            }
        }
    }

    return raw_infohash;
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

fn ascii_to_nibble(ascii: u8) -> u8 {
    let lower = ascii | 0b0010_0000; // set the 6th bit to lowercase it

    // A-F
    if lower > 0x60 {
        return lower - 0x57;
    } else {
        return lower - 0x30;
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
    fn is_legit_v2() {
        // All the extra bytes will be 0x00
        assert_eq!(*b"\x41\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00", url_encoded_to_raw_u8("A"));
        assert_eq!(*b"\x25\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00", url_encoded_to_raw_u8("%25"));
        assert_eq!(*b"\x43\x20\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00", url_encoded_to_raw_u8("C%20"));
        assert_eq!(*b"\xFF\x43\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00", url_encoded_to_raw_u8("%FFC"));
        assert_eq!(*b"\x41\x42\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00", url_encoded_to_raw_u8("%41B"));
        assert_eq!(*b"\x42\x41\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00", url_encoded_to_raw_u8("B%41"));
        assert_eq!(*b"\x42\x41\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00", url_encoded_to_raw_u8("BA"));
        assert_eq!(*b"\x41\x42\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00", url_encoded_to_raw_u8("%41%42"));

        // TODO: Add some tests for bad infohash
        // e.g. "%A" <- only 1 char following percent
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
    #[test]
    fn test_ascii_to_nibble(){
        assert_eq!(ascii_to_nibble(b'A'), 0x0A);
        assert_eq!(ascii_to_nibble(b'F'), 0x0F);
        assert_eq!(ascii_to_nibble(b'0'), 0x00);
        assert_eq!(ascii_to_nibble(b'9'), 0x09);
    }
}
