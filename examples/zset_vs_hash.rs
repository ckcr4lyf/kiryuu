/**
 * Tryingt to find at how many members is a ZSET less efficient than a HASH, WITH HEXPIRE.
 */


fn main(){
    let mut r = redis::Client::open("redis://127.0.0.1:1337").unwrap().get_connection().expect("failed to get connection");

    const ZSET_KEY: &[u8; 10] = b"ZSETAAAAAA";
    const HASH_KEY: &[u8; 10] = b"HASHONLYAA";
    const HASH_HEXPIRE_KEY: &[u8; 10] = b"HASHEXPIRE";

    // rand::thread_rng().gen();

    // TODO


}

