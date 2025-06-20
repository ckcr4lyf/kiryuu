use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;
use redis::Commands;

/**
 * Tryingt to find at how many members is a ZSET less efficient than a HASH, WITH HEXPIRE.
 */


fn main(){
    let mut r = redis::Client::open("redis://127.0.0.1:1337").unwrap().get_connection().expect("failed to get connection");

    const ZSET_KEY: &[u8; 10] = b"ZSETAAAAAA";
    const HASH_KEY: &[u8; 10] = b"HASHONLYAA";
    const HASH_HEXPIRE_KEY: &[u8; 10] = b"HASHEXPIRE";

    let time_now = SystemTime::now().duration_since(UNIX_EPOCH).expect("UHOH");
    let time_now_ms: i64 = i64::try_from(time_now.as_secs()).expect("UHOH");

    const TWENTY_FOUR_HOURS: i64 = 60 * 60 * 24;

    // rand::thread_rng().gen();
    for i in 0..200 {
        let fake_infohash: [u8; 20] = rand::thread_rng().gen();
        let () = r.zadd(ZSET_KEY, &fake_infohash, time_now_ms + TWENTY_FOUR_HOURS).expect("Failed to ZADD");
    }
    // TODO


}

