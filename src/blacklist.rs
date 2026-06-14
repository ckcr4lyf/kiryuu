use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub enum Action {
    Block,
    Redirect(String),
}

pub struct Blacklist {
    entries: HashMap<[u8; 20], Action>,
}

impl Blacklist {
    pub fn new() -> Self {
        Blacklist {
            entries: HashMap::new(),
        }
    }

    pub fn lookup(&self, hash: &[u8; 20]) -> Option<&Action> {
        self.entries.get(hash)
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
    let mut entries = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((hash_str, rest)) = line.split_once(',') {
            let url = rest.trim();
            if let Some(bytes) = hex_to_bytes(hash_str.trim()) {
                entries.insert(bytes, Action::Redirect(url.to_string()));
            }
        } else if let Some(bytes) = hex_to_bytes(line) {
            entries.insert(bytes, Action::Block);
        }
    }

    Ok(Blacklist { entries })
}
