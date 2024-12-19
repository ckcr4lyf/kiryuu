use redis::Commands;

fn main(){
    let mut c = redis::Client::open("redis://127.0.0.1:6379").unwrap().get_connection().expect("failed to get connection");
    let mut c2 = redis::Client::open("redis://127.0.0.1:6379").unwrap().get_connection().expect("failed to get connection");

    let mut keys = c.scan::<Vec<u8>>().expect("fail to get iterator to scan");

    while let Some(key) = keys.next() {
        // we know by design the hashes should have keysize=22
        if key.len() != 22 {
            continue;
        }

        // now check if there is a TTL set, if not we will EXPIRE it in 31 minutes
        match c2.ttl(&key) {
            Ok(-1) => {
                c2.expire(&key, 31 * 60).expect("failed to set expiry")
            },
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to get TTL: {}", e)
            }
        }
    }
}