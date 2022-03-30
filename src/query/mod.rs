mod tests;

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

// fn peer_bytes_to_str

pub fn announce_reply(seeders_count: usize, leechers_count: usize, seeders: Vec<Vec<u8>>, leechers: Vec<Vec<u8>>) -> Vec<u8> {
    // This is the number of peers in the response, not total peer count
    let peers_length = seeders.len() + leechers.len();

    let response_body_string = "d8:completei".to_string() 
    + &seeders_count.to_string()
    + &"e10:incompletei".to_string() 
    + &leechers_count.to_string()
    + &"e8:intervali1800e12:min intervali1800e5:peers".to_string()
    + &(peers_length * 6).to_string();

    let response_body: Vec<u8> = [response_body_string.into_bytes(), seeders.concat(), leechers.concat(), "e".as_bytes().to_vec()].concat() ;

    return response_body;
}
