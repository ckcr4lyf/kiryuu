use std::collections::HashMap;

use redis::Commands;

fn main(){
    let mut r_old = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut dupe = r_old.clone();
    let mut r_new = redis::Client::open("redis://127.0.0.1:6363").unwrap();

    // All torrents are in the ZSET TORRENTS
    // Migrate those...
    // migrate_torrrents_zset(&mut r_old, &mut r_new);

    // for the others we need to scan
    let mut x = r_old.scan::<String>().expect("fail to scan");

    let mut outer = 0;
    let mut count = 0;
    loop {
        // let a = 
        let mut inner = 0;
        if let Some(element) = x.next() {
            // println!("Got key: {}", element);

            // based on design of kiryuu:
            // If the key is 40 characters, then its a hash
            // If it is more, then it is likely a ZSET (`_seeders` or `_leechers`)
            // println!("New scan loop");
            if element.len() == 40 {
                
                let content: HashMap<String, String> = dupe.hgetall(&element).expect("fail to get it");

                if inner == 0 {
                    println!("Got key: {}", element);
                    println!("We got {:?}", content);
                }

                inner += 1;
                
                migrate_hash(&mut r_new, &element, &content);
                count += 1;
                // break;
            }
        } else {
            println!("Scan is complete")
        }

        outer += 1;
        println!("{} scans done!", outer);

        if outer == 100000 {
            break;
        }
    }

    println!("Did total: {}", count);
}

fn migrate_hash(new_server: &mut redis::Client, key: &str, data: &HashMap<String, String>){
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