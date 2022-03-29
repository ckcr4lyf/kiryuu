use serde_qs as qs;
use serde::Deserialize;

use crate::byte_functions;

#[derive(Debug, Deserialize)]
pub struct AReq {
    pub port: u16,
    pub info_hash: String,

    /// The amount of bytes the client has left to download
    /// for the purposes of a public tracker, the magnitude is insignificant
    /// what we care about is zero/non-zero , since it tells use if they are:
    /// zero left - seeder
    /// non-zero left - leecher
    left: String,

    pub event: Option<String>,
}

#[derive(Debug)]
pub struct PeerInfo {
    pub ip_port: Vec<u8>,
    pub info_hash: String,
    pub is_seeding: bool,
    pub event: Option<String>,
}

pub enum QueryError {
    ParseFailure,
    Custom(String),
}

// Allows us to use `?` postfix and wrap to QueryError
impl From<serde_qs::Error> for QueryError {
    fn from(_: serde_qs::Error) -> Self {
        return QueryError::ParseFailure;
    }
}

pub fn parse_announce(ip_str: &str, query: &str) -> Result<PeerInfo, QueryError> {
    // println!("The query is {}", query);
    let parsed: AReq = qs::from_str(query)?;
    // println!("Parsed it is {:?}", parsed);

    let hex_str_info_hash = byte_functions::url_encoded_to_hex(&parsed.info_hash);

    if hex_str_info_hash.len() != 40 {
        return Err(QueryError::Custom("Infohash was not 20 bytes".to_string()));
    }

    let is_seeding = match parsed.left.as_str() {
        "0" => true,
        _ => false,
    };

    return Ok(PeerInfo{
        ip_port: byte_functions::ip_str_port_u16_to_bytes(ip_str, parsed.port),
        info_hash: hex_str_info_hash,
        is_seeding,
        event: parsed.event,
    });
}