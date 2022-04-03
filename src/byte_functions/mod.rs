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

pub fn url_encoded_to_hex_v2(urlenc: &str) -> String {
    let mut hex_str = String::with_capacity(40);
    let mut chit = urlenc.chars();

    loop {
        match chit.next() {
            Some(c) => {
                match c {
                    '%' => {
                        let c1 = chit.next().expect("bruvva");
                        hex_str.push(c1.to_ascii_lowercase());
                        let c2 = chit.next().expect("bruvva");
                        hex_str.push(c2.to_ascii_lowercase());
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

pub fn url_encoded_to_hex_v4(urlenc: &str) -> String {
    
    let mut ihash: [char; 40] = ['0'; 40];
    let mut chit= urlenc.chars();
    let mut i = 0;


    while i < 40 {
        match chit.next() {
            Some(c) => {
                match c {
                    '%' => {
                        let c1 = chit.next().expect("bruvva");
                        ihash[i] = c1;
                        let c2 = chit.next().expect("bruvva");
                        ihash[i+1] = c2;
                        i += 2;
                    },
                    v => {
                        let mut myc = to_hex_op(v).chars();
                        ihash[i] = myc.next().expect("imp");
                        ihash[i+1] = myc.next().expect("imp");
                        i += 2;
                    }
                }
            },
            None => break
        }
    }

    return ihash.iter().collect();
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
fn get_lowercase(chr: char) -> char {
    match chr {
        'A' => 'a',
        'B' => 'b',
        'C' => 'c',
        'D' => 'd',
        'E' => 'e',
        'F' => 'f',
        'G' => 'g',
        'H' => 'h',
        'I' => 'i',
        'J' => 'j',
        'K' => 'k',
        'L' => 'l',
        'M' => 'm',
        'N' => 'n',
        'O' => 'o',
        'P' => 'p',
        'Q' => 'q',
        'R' => 'r',
        'S' => 's',
        'T' => 't',
        'U' => 'u',
        'V' => 'v',
        'W' => 'w',
        'X' => 'x',
        _ => 'a',
    }
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
        'b' => 11,
        'c' => 12,
        'd' => 13,
        'e' => 14,
        'f' => 15,
        _ => 0, // Fucked up but yolo for now
    }
}
