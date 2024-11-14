use redis::Commands;

fn main(){
    let mut r_old = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut r_new = redis::Client::open("redis://127.0.0.1:6363").unwrap();

    // All torrents are in the ZSET TORRENTS
    let all_torrents: Vec<(Vec<u8>, f64)> = r_old.zrangebyscore_withscores("TORRENTS", "-inf", "+inf").expect("failed to ZRANGEBYSCORE");

    println!("Got {} torrents", all_torrents.len());

    for i in 0..all_torrents.len() {
        if i % 10000 == 0 {
            println!("Currently doing torrent #{}", i + 1);
        }

        let raw_infohash = hex::decode(&all_torrents[i].0).expect("Failed to decode");
        r_new.zadd::<&str, f64, &Vec<u8>, ()>("TORRENTS", &raw_infohash, all_torrents[i].1).expect("failed to zadd");
    }

    // unsafe  {
    //     let tuple = &all_torrents[0];
    //     println!("First guy: {:?}", std::str::from_utf8_unchecked(&tuple.0));
    //     let decoded = hex::decode(&tuple.0).expect("failed to decode");
    //     println!("First raw: {:2X?}", decoded);
    //     println!("Score is: {:?}", tuple.1);
    //     r_new.zadd::<&str, f64, &Vec<u8>, ()>("TORRENTS", &decoded, tuple.1).expect("failed to zadd");
    // }
}