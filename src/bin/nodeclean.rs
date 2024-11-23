use std::time::{SystemTime, UNIX_EPOCH};

use kiryuu::byte_functions::types;
use log::info;
use redis::Commands;

const TORRENTS_KEY: &[u8; 8] = b"TORRENTS";
const THIRTY_ONE_MINUTES: i64 = 60 * 31 * 1000;

fn main(){
    env_logger::init();
    info!("starting");
    let mut r_old = redis::Client::open("redis://127.0.0.1:6379").unwrap().get_connection().expect("failed to get connection");
    let mut r_old_2 = redis::Client::open("redis://127.0.0.1:6379").unwrap().get_connection().expect("failed to get connection");

    let time_now = SystemTime::now().duration_since(UNIX_EPOCH).expect("fucked up");
    let time_now_ms: i64 = i64::try_from(time_now.as_millis()).expect("fucc");
    let max_limit = time_now_ms - THIRTY_ONE_MINUTES;

    let mut offset: isize = 0;
    let mut seeder_key: [u8; 48] = *b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA_seeders";
    let mut leecher_key: [u8; 49] = *b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA_leechers";

    loop {
        let oldies: Vec<Vec<u8>> = r_old.zrangebyscore_limit(TORRENTS_KEY, 0, max_limit, offset, 1000).expect("failed to get");

        if oldies.len() == 0 {
            info!("the end!");
            break;
        }

        let mut pipeline = redis::pipe();

        for oldie in oldies {
            seeder_key[..40].copy_from_slice(&oldie);
            leecher_key[..40].copy_from_slice(&oldie);
            pipeline.del(&oldie);
            pipeline.del(types::RawVal(seeder_key));
            pipeline.del(types::RawVal(leecher_key));
        }

        pipeline.execute(&mut r_old);
        offset += 1000;
    }

    info!("Done. offset: {}", offset);
}

fn overwrite_keys(infohash: &[u8], key: &mut [u8]){
    for i in 0..40 {
        key[i] = infohash[i];
    }
}