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
    pub ip_port: [u8; 6],
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

pub fn parse_announce(ip_addr: &std::net::Ipv4Addr, query: &[u8]) -> Result<PeerInfo, QueryError> {

    // Solution: manually parse & encoded infohash from `&info_hash=.........&xyz=.....
    let parsed: AReq = qs::from_bytes(query)?;

    // let hex_str_info_hash = "XD";
    let hex_str_info_hash = byte_functions::url_encoded_to_hex(&parsed.info_hash);

    if hex_str_info_hash.len() != 40 {
        return Err(QueryError::Custom("Infohash was not 20 bytes".to_string()));
    }

    let is_seeding = match parsed.left.as_str() {
        "0" => true,
        _ => false,
    };

    return Ok(PeerInfo{
        ip_port: byte_functions::ip_str_port_u16_to_bytes(ip_addr, parsed.port),
        info_hash: hex_str_info_hash.to_string(),
        is_seeding,
        event: parsed.event,
    });
}

// fn peer_bytes_to_str

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