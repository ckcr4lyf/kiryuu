use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct Blacklist {
    hashes: HashSet<[u8; 20]>,
}

impl Blacklist {
    pub fn contains(&self, hash: &[u8; 20]) -> bool {
        self.hashes.contains(hash)
    }
}

fn hex_to_bytes(hex: &str) -> Option<[u8; 20]> {
    if hex.len() != 40 {
        return None;
    }
    let mut bytes = [0u8; 20];
    let hex_bytes = hex.as_bytes();
    for i in 0..20 {
        let high = ascii_to_nibble(hex_bytes[i * 2])?;
        let low = ascii_to_nibble(hex_bytes[i * 2 + 1])?;
        bytes[i] = (high << 4) | low;
    }
    Some(bytes)
}

#[inline(always)]
fn ascii_to_nibble(ascii: u8) -> Option<u8> {
    match ascii {
        b'0'..=b'9' => Some(ascii - b'0'),
        b'a'..=b'f' => Some(ascii - b'a' + 10),
        b'A'..=b'F' => Some(ascii - b'A' + 10),
        _ => None,
    }
}

pub fn load_blacklist(path: &str) -> std::io::Result<Blacklist> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut hashes = HashSet::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(bytes) = hex_to_bytes(line) {
            hashes.insert(bytes);
        }
    }

    Ok(Blacklist { hashes })
}
