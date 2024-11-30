use std::time::Instant;

use kiryuu::byte_functions::types::RawVal;
use rand::Rng;
use redis::AsyncCommands;

#[actix_web::main]
async fn main(){

    let mut redis_connection = redis::Client::open("redis://127.0.0.1:6379").unwrap().get_multiplexed_async_connection().await.unwrap();
    println!("connected to redis");

    // let key = RawVal::<22>(*b"AAAAAAAAAAAAAAAAAAAAAE");
    let b: [u8; 22] = rand::thread_rng().gen();
    let key = RawVal::<22>(b);

    // insert 10000 peers
    let mut base = Instant::now();

    for i in 0..10 {
        let rando: [u8; 6] = rand::thread_rng().gen();
        let () = redis_connection.hset(&key, &rando, 1u8).await.expect("failed to hset");
    }

    println!("inserted in {}ms", base.elapsed().as_millis());

    base = Instant::now();
    let () = redis_connection.hkeys(&key).await.expect("failed to hkeys");
    println!("got cache hkeys in {}us", base.elapsed().as_micros());
    
    base = Instant::now();
    let (dudes, count) = kiryuu::db::get_hash_keys_scan_stack(&mut redis_connection, &key, 50).await;
    println!("got scan-u8u8 in {}us", base.elapsed().as_micros());
    base = Instant::now();
    let dudes = kiryuu::db::get_hash_keys_scan(&mut redis_connection, &key, 50).await;
    println!("got scan-vecu8 in {}us", base.elapsed().as_micros());

    base = Instant::now();
    let k: Vec<Vec<u8>> = redis_connection.hkeys(&key).await.expect("failed to hkeys");
    println!("got hkeys-vecvec in {}us", base.elapsed().as_micros());

    base = Instant::now();
    let k: Vec<[u8; 6]> = redis_connection.hkeys(&key).await.expect("failed to hkeys");
    println!("got hkeys-vecu8 in {}us", base.elapsed().as_micros());
    
}