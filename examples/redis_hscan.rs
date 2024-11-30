use kiryuu::byte_functions::types::RawVal;
use rand::Rng;
use redis::AsyncCommands;

#[actix_web::main]
async fn main(){

    let mut redis_connection = redis::Client::open("redis://127.0.0.1:6379").unwrap().get_multiplexed_async_connection().await.unwrap();
    println!("connected to redis");

    let key = RawVal::<22>(*b"AAAAAAAAAAAAAAAAAAAAAA");

    // insert 1000 peers
    for i in 0..1000 {
        let rando: [u8; 6] = rand::thread_rng().gen();
        let () = redis_connection.hset(&key, &rando, 1u8).await.expect("failed to hset");
    }

    

}