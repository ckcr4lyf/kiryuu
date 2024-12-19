use redis::Commands;

fn main(){
    let mut c = redis::Client::open("redis://127.0.0.1:6379").unwrap().get_connection().expect("failed to get connection");
    let mut c2 = redis::Client::open("redis://127.0.0.1:6379").unwrap().get_connection().expect("failed to get connection");

    let mut keys = c.scan::<Vec<u8>>().expect("fail to get iterator to scan");

    while let Some(key) = keys.next() {
        // we know by design the hashes should have keysize=22
        if key.len() != 20 {
            continue;
        }

        let () = c2.del(&key).expect("failed to delete key");
    }
}