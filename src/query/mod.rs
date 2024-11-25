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

pub enum Event {
    Unknown,
    Stopped,
    Completed,
}

pub struct PeerInfo {
    pub ip_port: [u8; 6],
    pub info_hash: byte_functions::types::RawVal<20>,
    pub is_seeding: bool,
    pub event: Event
}

pub enum QueryError {
    ParseFailure,
    InvalidInfohash,
}

// Allows us to use `?` postfix and wrap to QueryError
impl From<serde_qs::Error> for QueryError {
    fn from(_: serde_qs::Error) -> Self {
        return QueryError::ParseFailure;
    }
}

pub fn parse_announce(ip_addr: &std::net::Ipv4Addr, query: &[u8]) -> Result<PeerInfo, QueryError> {
    let parsed: AReq = qs::from_bytes(query)?;
    let raw_infohash = byte_functions::url_encoded_to_raw_u8(&parsed.info_hash);

    let is_seeding = match parsed.left.as_str() {
        "0" => true,
        _ => false,
    };

    let announce_event = if let Some(ref event) = parsed.event {
        match event.as_str() {
            "stopped" => Event::Stopped,
            "completed" => Event::Completed,
            _ => Event::Unknown,
        }
    } else {
        Event::Unknown
    };

    return Ok(PeerInfo{
        ip_port: byte_functions::ip_str_port_u16_to_bytes(ip_addr, parsed.port),
        info_hash: byte_functions::types::RawVal(raw_infohash),
        is_seeding,
        event: announce_event,
    });
}

pub fn announce_reply(seeders_count: i64, leechers_count: i64, seeders: &[Vec<u8>], leechers: &[Vec<u8>]) -> Vec<u8> {
    // This is the number of peers in the response, not total peer count
    let peers_length = seeders.len() + leechers.len();

    let response_body_string = "d8:completei".to_string() 
    + &seeders_count.to_string()
    + &"e10:incompletei".to_string() 
    + &leechers_count.to_string()
    + &"e8:intervali1800e12:min intervali1800e5:peers".to_string()
    + &(peers_length * 6).to_string()
    + &":";

    let response_body: Vec<u8> = [response_body_string.into_bytes(), seeders.concat(), leechers.concat(), "e".as_bytes().to_vec()].concat() ;

    return response_body;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_legit(){
        let bytes: Vec<u8> = Vec::from([1, 2, 3, 4]);
        let _no_bytes: Vec<u8> = Vec::from([]);
    
        let mut p1: Vec<Vec<u8>> = Vec::new();
        p1.push(bytes);
        
        let p2: Vec<Vec<u8>> = Vec::new();
        // p2.push(no_bytes);
    
        // TODO: Actually implement a test here...
        let gg = announce_reply(1, 2, &p1, &p2);
        println!("GG is {:?}", gg);
    }
}