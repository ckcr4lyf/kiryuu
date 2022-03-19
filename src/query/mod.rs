use serde_qs as qs;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AReq {
    xd: String,
    pub port: u16,
}

pub fn parse_announce(query: &str) -> Result<AReq, serde_qs::Error> {
    println!("The query is {}", query);
    let parsed: AReq = qs::from_str(query)?;
    println!("Parsed it is {:?}", parsed);
    return Ok(parsed);
}