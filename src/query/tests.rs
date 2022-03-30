#[cfg(test)]
use crate::query;

#[test]
fn is_legit(){
    let bytes: Vec<u8> = Vec::from([1, 2, 3, 4]);
    let no_bytes: Vec<u8> = Vec::from([]);

    let mut p1: Vec<Vec<u8>> = Vec::new();
    p1.push(bytes);
    
    let mut p2: Vec<Vec<u8>> = Vec::new();
    // p2.push(no_bytes);

    let gg = query::announce_reply(1, 2, p1, p2);
    println!("GG is {:?}", gg);
}