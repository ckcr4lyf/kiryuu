mod tests;

use std::str::from_utf8_unchecked;

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

pub fn xd(x: &str) -> String {

    let mut hex_str_bytes: [u8; 40] = [0; 40];

    let mut pos_x = 0;
    let mut pos_hex_str = 0;
    let raw = x.as_bytes();
    let max = raw.len();

    while pos_x < max {
        match raw[pos_x] {
            // % , meaning the next two char can be used as is
            // well, we lowercase it
            0x25 => {
                hex_str_bytes[pos_hex_str] = raw[pos_x+1];
                hex_str_bytes[pos_hex_str+1] = raw[pos_x+2];
                pos_hex_str += 2; // We have done two bytes in the hex str's bytes
                pos_x += 3;
            },
            non_pc => {
                // This byte is the actual info hash byte
                // We need its text version
                // e.g. A => 0x41 => "41" => [0x34, 0x31]
                // Get high 4 bits (i,e 0x4_)
                let left = (0b11110000 & non_pc) >> 4;

                // Get low 4 bits (i,e 0x_1)
                let right = 0b00001111 & non_pc;

                hex_str_bytes[pos_hex_str] = HEX_CHAR_MAP[left as usize];
                hex_str_bytes[pos_hex_str+1] = HEX_CHAR_MAP[right as usize];
                pos_hex_str += 2;
                pos_x += 1;
            }
        }
    }

    let str_val = unsafe {
        from_utf8_unchecked(&hex_str_bytes[0..pos_hex_str])
    };
    
    return str_val.to_owned();
}

pub fn xd2(x: &str) -> [u8; 40] {

    let mut hex_str_bytes: [u8; 40] = [0; 40];

    let mut pos_x = 0;
    let mut pos_hex_str = 0;
    let raw = x.as_bytes();
    let max = raw.len();

    while pos_x < max {
        match raw[pos_x] {
            // % , meaning the next two char can be used as is
            // well, we lowercase it
            0x25 => {
                hex_str_bytes[pos_hex_str] = raw[pos_x+1];
                hex_str_bytes[pos_hex_str+1] = raw[pos_x+2];
                pos_hex_str += 2; // We have done two bytes in the hex str's bytes
                pos_x += 3;
            },
            non_pc => {
                // This byte is the actual info hash byte
                // We need its text version
                // e.g. A => 0x41 => "41" => [0x34, 0x31]
                let temp = to_hex_op_v0(&non_pc);
                hex_str_bytes[pos_hex_str] = temp[0];
                hex_str_bytes[pos_hex_str+1] = temp[1];
                pos_hex_str += 2;
                pos_x += 1;
            }
        }
    }

    return hex_str_bytes;
}

const HEX_CHAR_MAP: [u8; 16] = [0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46];

pub fn xd3(x: &str) -> [u8; 40] {

    let mut hex_str_bytes: [u8; 40] = [0; 40];

    let mut pos_x = 0;
    let mut pos_hex_str = 0;
    let raw = x.as_bytes();
    let max = raw.len();

    while pos_x < max {
        match raw[pos_x] {
            // % , meaning the next two char can be used as is
            // well, we lowercase it
            0x25 => {
                hex_str_bytes[pos_hex_str] = raw[pos_x+1];
                hex_str_bytes[pos_hex_str+1] = raw[pos_x+2];
                pos_hex_str += 2; // We have done two bytes in the hex str's bytes
                pos_x += 3;
            },
            non_pc => {
                // This byte is the actual info hash byte
                // We need its text version
                // e.g. A => 0x41 => "41" => [0x34, 0x31]

                // Get high 4 bits
                let left = (0b11110000 & non_pc) >> 4;

                // Get low 4 bits
                let right = 0b00001111 & non_pc;

                hex_str_bytes[pos_hex_str] = HEX_CHAR_MAP[left as usize];
                hex_str_bytes[pos_hex_str+1] = HEX_CHAR_MAP[right as usize];
                pos_hex_str += 2;
                pos_x += 1;
            }
        }
    }

    return hex_str_bytes;
}

const ULTRA_MAP: [[u8; 2]; 256] = [[0x30,0x30], [0x30,0x31], [0x30,0x32], [0x30,0x33], [0x30,0x34], [0x30,0x35], [0x30,0x36], [0x30,0x37], [0x30,0x38], [0x30,0x39], [0x30,0x41], [0x30,0x42], [0x30,0x43], [0x30,0x44], [0x30,0x45], [0x30,0x46], [0x31,0x30], [0x31,0x31], [0x31,0x32], [0x31,0x33], [0x31,0x34], [0x31,0x35], [0x31,0x36], [0x31,0x37], [0x31,0x38], [0x31,0x39], [0x31,0x41], [0x31,0x42], [0x31,0x43], [0x31,0x44], [0x31,0x45], [0x31,0x46], [0x32,0x30], [0x32,0x31], [0x32,0x32], [0x32,0x33], [0x32,0x34], [0x32,0x35], [0x32,0x36], [0x32,0x37], [0x32,0x38], [0x32,0x39], [0x32,0x41], [0x32,0x42], [0x32,0x43], [0x32,0x44], [0x32,0x45], [0x32,0x46], [0x33,0x30], [0x33,0x31], [0x33,0x32], [0x33,0x33], [0x33,0x34], [0x33,0x35], [0x33,0x36], [0x33,0x37], [0x33,0x38], [0x33,0x39], [0x33,0x41], [0x33,0x42], [0x33,0x43], [0x33,0x44], [0x33,0x45], [0x33,0x46], [0x34,0x30], [0x34,0x31], [0x34,0x32], [0x34,0x33], [0x34,0x34], [0x34,0x35], [0x34,0x36], [0x34,0x37], [0x34,0x38], [0x34,0x39], [0x34,0x41], [0x34,0x42], [0x34,0x43], [0x34,0x44], [0x34,0x45], [0x34,0x46], [0x35,0x30], [0x35,0x31], [0x35,0x32], [0x35,0x33], [0x35,0x34], [0x35,0x35], [0x35,0x36], [0x35,0x37], [0x35,0x38], [0x35,0x39], [0x35,0x41], [0x35,0x42], [0x35,0x43], [0x35,0x44], [0x35,0x45], [0x35,0x46], [0x36,0x30], [0x36,0x31], [0x36,0x32], [0x36,0x33], [0x36,0x34], [0x36,0x35], [0x36,0x36], [0x36,0x37], [0x36,0x38], [0x36,0x39], [0x36,0x41], [0x36,0x42], [0x36,0x43], [0x36,0x44], [0x36,0x45], [0x36,0x46], [0x37,0x30], [0x37,0x31], [0x37,0x32], [0x37,0x33], [0x37,0x34], [0x37,0x35], [0x37,0x36], [0x37,0x37], [0x37,0x38], [0x37,0x39], [0x37,0x41], [0x37,0x42], [0x37,0x43], [0x37,0x44], [0x37,0x45], [0x37,0x46], [0x38,0x30], [0x38,0x31], [0x38,0x32], [0x38,0x33], [0x38,0x34], [0x38,0x35], [0x38,0x36], [0x38,0x37], [0x38,0x38], [0x38,0x39], [0x38,0x41], [0x38,0x42], [0x38,0x43], [0x38,0x44], [0x38,0x45], [0x38,0x46], [0x39,0x30], [0x39,0x31], [0x39,0x32], [0x39,0x33], [0x39,0x34], [0x39,0x35], [0x39,0x36], [0x39,0x37], [0x39,0x38], [0x39,0x39], [0x39,0x41], [0x39,0x42], [0x39,0x43], [0x39,0x44], [0x39,0x45], [0x39,0x46], [0x41,0x30], [0x41,0x31], [0x41,0x32], [0x41,0x33], [0x41,0x34], [0x41,0x35], [0x41,0x36], [0x41,0x37], [0x41,0x38], [0x41,0x39], [0x41,0x41], [0x41,0x42], [0x41,0x43], [0x41,0x44], [0x41,0x45], [0x41,0x46], [0x42,0x30], [0x42,0x31], [0x42,0x32], [0x42,0x33], [0x42,0x34], [0x42,0x35], [0x42,0x36], [0x42,0x37], [0x42,0x38], [0x42,0x39], [0x42,0x41], [0x42,0x42], [0x42,0x43], [0x42,0x44], [0x42,0x45], [0x42,0x46], [0x43,0x30], [0x43,0x31], [0x43,0x32], [0x43,0x33], [0x43,0x34], [0x43,0x35], [0x43,0x36], [0x43,0x37], [0x43,0x38], [0x43,0x39], [0x43,0x41], [0x43,0x42], [0x43,0x43], [0x43,0x44], [0x43,0x45], [0x43,0x46], [0x44,0x30], [0x44,0x31], [0x44,0x32], [0x44,0x33], [0x44,0x34], [0x44,0x35], [0x44,0x36], [0x44,0x37], [0x44,0x38], [0x44,0x39], [0x44,0x41], [0x44,0x42], [0x44,0x43], [0x44,0x44], [0x44,0x45], [0x44,0x46], [0x45,0x30], [0x45,0x31], [0x45,0x32], [0x45,0x33], [0x45,0x34], [0x45,0x35], [0x45,0x36], [0x45,0x37], [0x45,0x38], [0x45,0x39], [0x45,0x41], [0x45,0x42], [0x45,0x43], [0x45,0x44], [0x45,0x45], [0x45,0x46], [0x46,0x30], [0x46,0x31], [0x46,0x32], [0x46,0x33], [0x46,0x34], [0x46,0x35], [0x46,0x36], [0x46,0x37], [0x46,0x38], [0x46,0x39], [0x46,0x41], [0x46,0x42], [0x46,0x43], [0x46,0x44], [0x46,0x45], [0x46,0x46]];

pub fn xd4(x: &str) -> [u8; 40] {

    let mut hex_str_bytes: [u8; 40] = [0; 40];

    let mut pos_x = 0;
    let mut pos_hex_str = 0;
    let raw = x.as_bytes();
    let max = raw.len();

    while pos_x < max {
        match raw[pos_x] {
            // % , meaning the next two char can be used as is
            // well, we lowercase it
            0x25 => {
                hex_str_bytes[pos_hex_str] = raw[pos_x+1];
                hex_str_bytes[pos_hex_str+1] = raw[pos_x+2];
                pos_hex_str += 2; // We have done two bytes in the hex str's bytes
                pos_x += 3;
            },
            non_pc => {
                // This byte is the actual info hash byte
                // We need its text version
                // e.g. A => 0x41 => "41" => [0x34, 0x31]

                // Get high 4 bits
                // let left = (0b11110000 & non_pc) >> 4;

                // // Get low 4 bits
                // let right = 0b00001111 & non_pc;

                hex_str_bytes[pos_hex_str] = ULTRA_MAP[non_pc as usize][0];
                hex_str_bytes[pos_hex_str+1] = ULTRA_MAP[non_pc as usize][1];
                pos_hex_str += 2;
                pos_x += 1;
            }
        }
    }

    return hex_str_bytes;
}

pub fn url_encoded_to_hex_v2(urlenc: &str) -> String {
    let mut hex_str = String::with_capacity(40);
    let mut chit = urlenc.chars();

    loop {
        match chit.next() {
            Some(c) => {
                match c {
                    '%' => {
                        let c1 = chit.next().expect("bruvva");
                        // hex_str.push(c1.to_ascii_lowercase());
                        hex_str.push(get_lowercase(&c1));
                        let c2 = chit.next().expect("bruvva");
                        // hex_str.push(c2.to_ascii_lowercase());
                        hex_str.push(get_lowercase(&c2));
                    },
                    v => {
                        hex_str.push_str(to_hex_op(v));
                    }
                }
            },
            None => break
        }
    }

    return hex_str;
}

pub fn url_encoded_to_hex_v3(urlenc: &str) -> [u8; 20] {
    let mut bytes: [u8; 20] = [0; 20];
    let mut chit= urlenc.chars();
    let mut i = 0;


    while i < 20 {
        match chit.next() {
            Some(c) => {
                match c {
                    '%' => {
                        let c1 = chit.next().expect("bruvva");
                        let c2 = chit.next().expect("bruvva");
                        bytes[i] = two_char_to_byte(c1, c2);
                        i += 1;
                    },
                    v => {
                        bytes[i] = v as u8;
                        i += 1;
                    }
                }
            },
            None => break
        }
    }

    return bytes;
}

pub fn url_encoded_to_hex_v5(urlenc: &str) -> [u8; 20] {
    let mut bytes: [u8; 20] = [0; 20];
    let mut chit= urlenc.chars();
    let mut i = 0;


    while i < 20 {
        match chit.next() {
            Some(c) => {
                match c {
                    '%' => {
                        let c1 = chit.next().expect("bruvva");
                        let c2 = chit.next().expect("bruvva");
                        bytes[i] = two_char_to_byte_v2(c1, c2);
                        i += 1;
                    },
                    v => {
                        bytes[i] = v as u8;
                        i += 1;
                    }
                }
            },
            None => break
        }
    }

    return bytes;
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

// The URL has % encoded hex valyes
// Such as %C3. We want the literal "c3" for it
// But if it comes as %c3 , then, well, also "c3"
// So map uppercase to lower, lower as is, using match.
fn get_lowercase(chr: &char) -> char {
    match chr {
        'A' => 'a',
        'B' => 'b',
        'C' => 'c',
        'D' => 'd',
        'E' => 'e',
        'F' => 'f',
        'a' => 'a',
        'b' => 'b',
        'c' => 'c',
        'd' => 'd',
        'e' => 'e',
        'f' => 'f',
        _ => 'a', // should never happen
    }
}

// For a single 0-16 value (4-bit), return the ascii
// of the hex character that represents it
// e.g the number 11 => 0xB in hex => "B" => 0x42
fn half_byte_to_hex_char(actual: &u8) -> u8 {
    match actual {
        0x0 => 0x30, // ("0")
        0x1 => 0x31, // ("1")
        0x2 => 0x32, // ("2")
        0x3 => 0x33, // ("3")
        0x4 => 0x34, // ("4")
        0x5 => 0x35, // ("5")
        0x6 => 0x36, // ("6")
        0x7 => 0x37, // ("7")
        0x8 => 0x38, // ("8")
        0x9 => 0x39, // ("9")
        0xA => 0x41, // ("A")
        0xB => 0x42, // ("B")
        0xC => 0x43, // ("C")
        0xD => 0x44, // ("D")
        0xE => 0x45, // ("E")
        0xF => 0x46, // ("F")
        _ => panic!("impossible")
    }

    // Would a 16 element array lookup be quicker?? TBD...
}

// This will take in a whole byte, and return its
// hex string, which is two bytes
// e.g. 0x41 -> "41" = [0x34, 0x31]
fn to_hex_op_v0(actual: &u8) -> [u8; 2] {
    // Get high 4 bits
    let left = (0b11110000 & actual) >> 4;

    // Get low 4 bits
    let right = 0b00001111 & actual;

    // First character
    let left_char = half_byte_to_hex_char(&left);
    let right_char = half_byte_to_hex_char(&right);

    return [left_char, right_char];
}

fn to_hex_op(chr: char) -> &'static str {
    match chr {
        'a' => "61",
        'A' => "41",
        'b' => "62",
        'B' => "42",
        'c' => "63",
        'C' => "43",
        'd' => "64",
        'D' => "44",
        'e' => "65",
        'E' => "45",
        'f' => "66",
        'F' => "46",
        'g' => "67",
        'G' => "47",
        'h' => "68",
        'H' => "48",
        'i' => "69",
        'I' => "49",
        'j' => "6a",
        'J' => "4a",
        'k' => "6b",
        'K' => "4b",
        'l' => "6c",
        'L' => "4c",
        'm' => "6d",
        'M' => "4d",
        'n' => "6e",
        'N' => "4e",
        'o' => "6f",
        'O' => "4f",
        'p' => "70",
        'P' => "50",
        'q' => "71",
        'Q' => "51",
        'r' => "72",
        'R' => "52",
        's' => "73",
        'S' => "53",
        't' => "74",
        'T' => "54",
        'u' => "75",
        'U' => "55",
        'v' => "76",
        'V' => "56",
        'w' => "77",
        'W' => "57",
        'x' => "78",
        'X' => "58",
        'y' => "79",
        'Y' => "59",
        'z' => "7a",
        'Z' => "5a",
        '*' => "2a",
        '-' => "2d",
        '.' => "2e",
        '_' => "5f",
        _ => "00",
    }
}

fn two_char_to_byte(c1: char, c2: char) -> u8 {
    return to_hex_decimal(c1) * 16 + to_hex_decimal(c2);
}

fn to_hex_decimal(chr: char) -> u8 {
    match chr {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'a' => 10,
        'A' => 10,
        'b' => 11,
        'B' => 11,
        'c' => 12,
        'C' => 12,
        'd' => 13,
        'D' => 13,
        'e' => 14,
        'E' => 14,
        'f' => 15,
        'F' => 15,
        _ => 0, // Fucked up but yolo for now
    }
}

fn two_char_to_byte_v2(c1: char, c2: char) -> u8 {
    return to_hex_decimal_v2(c1) * 16 + to_hex_decimal_v2(c2);
}

fn to_hex_decimal_v2(chr: char) -> u8 {

    let x = chr as u8;
    let lower_4 = x & 0x0f;
    let is_letter = (x & 0x40) >> 6;
    
    return lower_4 + is_letter * 9;

}
