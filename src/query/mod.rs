use serde_qs as qs;
use serde::Deserialize;

use crate::byte_functions;

#[derive(Debug, Deserialize)]
pub struct AReq {
    pub port: u16,
    pub info_hash: String,
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

pub fn parse_announce(query: &str) -> Result<AReq, QueryError> {
    println!("The query is {}", query);
    let mut parsed: AReq = qs::from_str(query)?;
    println!("Parsed it is {:?}", parsed);

    parsed.info_hash = byte_functions::url_encoded_to_hex(&parsed.info_hash);

    if parsed.info_hash.len() != 40 {
        return Err(QueryError::Custom("Infohash was not 20 bytes".to_string()));
    }

    return Ok(parsed);
}