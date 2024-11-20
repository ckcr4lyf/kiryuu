use std::collections::HashMap;
use redis::Commands;

fn main(){
    let mut r_old = redis::Client::open("redis://127.0.0.1:6379").unwrap().get_connection().expect("failed to get connection");
    let mut r_old_2 = redis::Client::open("redis://127.0.0.1:6379").unwrap().get_connection().expect("failed to get connection");
    // let mut dupe = r_old.clone().get_connection().expect("fucc");
    let mut r_new = redis::Client::open("redis://127.0.0.1:6363").unwrap().get_connection().expect("failed to get connection for new");

    // All torrents are in the ZSET TORRENTS
    // Migrate those...
    // migrate_torrrents_zset(&mut r_old, &mut r_new);
    
    // For all the other keys we need to scan the keyspace
    // Get an iterator on it
    // We need `Vec<u8>` since the keys can technically be raw bytes as well
    let mut x = r_old.scan::<Vec<u8>>().expect("fail to scan");

    while let Some(element) = x.next() {
        // println!("Got key: {}", element);

        // based on design of kiryuu:
        // If the key is 40 characters, then its a hash
        // If it is more, then it is likely a ZSET (`_seeders` or `_leechers`)
        if element.len() == 40 {
            let content: HashMap<String, String> = r_old_2.hgetall(&element).expect("fail to get it");
            unsafe { println!("Got key: {}", String::from_utf8_unchecked(element.clone())); }
            println!("We got {:?}", content);
            // migrate_hash(&mut r_new, &element, &content);
        } else if element.len() < 40 {
            println!("Weird {:?}", element)
        } else if element.len() > 42 {
            // len more than 40, should be *_seeders or *_leechers
            unsafe { println!("Got key: {}", String::from_utf8_unchecked(element.clone())); }
            
            // construct the new key
            let mut new_key: Vec<u8> = vec![0u8; 20];
            hex::decode_to_slice(&element[0..40], &mut new_key).expect("failed to decode");
            new_key.push(b':');
            new_key.push(element[41]);
            println!("new key is {:02x?}", new_key);

            // migrate the ZSET
            let peers_with_scores: Vec<(Vec<u8>, f64)> = r_old_2.zrangebyscore_withscores(&element, "-inf", "+inf").expect("fail to get zset");
            println!("We got {:?}", peers_with_scores);
        }
    }
}

fn migrate_hash(new_server: &mut redis::Connection, key: &Vec<u8>, data: &HashMap<String, String>){
    let new_key = hex::decode(key).expect("failed to decode");
    // new_server.hset_multiple(new_key, data.);

    for (realkey, value) in data {
        let _: () = new_server.hset(&new_key, realkey, value).expect("fail to hset");
        // println!("We pogging with {} and {} into {:2X?}", realkey, value, new_key);
    }
    // new_server.hset(key, field, value)

    // println!("For {}, we set {:?}", key, data);
}


fn migrate_torrrents_zset(old_server: &mut redis::Client, new_server: &mut redis::Client){
    let all_torrents: Vec<(Vec<u8>, f64)> = old_server.zrangebyscore_withscores("TORRENTS", "-inf", "+inf").expect("failed to ZRANGEBYSCORE");

    println!("Got {} torrents", all_torrents.len());

    for i in 0..all_torrents.len() {
        if i % 10000 == 0 {
            println!("Currently doing torrent #{}", i + 1);
        }

        let raw_infohash = hex::decode(&all_torrents[i].0).expect("Failed to decode");
        new_server.zadd::<&str, f64, &Vec<u8>, ()>("TORRENTS", &raw_infohash, all_torrents[i].1).expect("failed to zadd");
    }
}