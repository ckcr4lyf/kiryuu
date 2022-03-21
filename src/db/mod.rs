use redis::{self, Commands};

pub fn get_guys() {

    let client = redis::Client::open("redis://127.0.0.1").unwrap();
    let mut con = client.get_connection().unwrap();

    let guys: Vec<Vec<u8>> = con.zrangebyscore("abc", 100, 200).unwrap();

    println!("Guys are {:?}", guys);
}