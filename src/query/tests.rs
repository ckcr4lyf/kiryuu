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

    let gg = query::announce_reply(1, 2, &p1, &p2);
    println!("GG is {:?}", gg);
}


#[test]
fn parseCompare(){
    for _ in 0..1000000 {
        query::parse_announce("127.0.0.1", b"port=3333&info_hash=%25DD%2500%25D2%251CuDA%25AAL%25B6J%251E%25A7z%252CvFAR%25C3&left=0");
        query::parse_announce_u8("127.0.0.1", b"port=3333&info_hash=%25DD%2500%25D2%251CuDA%25AAL%25B6J%251E%25A7z%252CvFAR%25C3&left=0");
    }
}