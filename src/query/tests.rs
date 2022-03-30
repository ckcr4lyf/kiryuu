#[cfg(test)]
use crate::query;

use serde::Deserialize;

#[test]
fn is_legit(){
    let bytes: Vec<u8> = Vec::from([1, 2, 3, 4]);
    let no_bytes: Vec<u8> = Vec::from([]);

    let mut p1: Vec<Vec<u8>> = Vec::new();
    p1.push(bytes);
    
    let mut p2: Vec<Vec<u8>> = Vec::new();
    // p2.push(no_bytes);

    let gg = query::announce_reply(1, 2, &p1, &p2);
    println!("GG is {:?}", gg);
}

#[test]
fn bruvva(){
    let meal = vec![
    ("bread".to_owned(), "baguette".to_owned()),
    ("cheese".to_owned(), "comté".to_owned()),
    ("meat".to_owned(), "ham".to_owned()),
    ("fat".to_owned(), "butter".to_owned()),
];

#[derive(Debug, Deserialize)]
struct gg {
    pub bread: String,
}

    let x: Vec<(String, Vec<u8>)> = serde_urlencoded::from_bytes(b"bread=%c3XD").unwrap();

    // println!("X is {:?}", x.bread.as_bytes());
    println!("X is {:?}", x);
}