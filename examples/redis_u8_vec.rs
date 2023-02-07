use redis::{self, Commands};


fn main(){
    let mut r_client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut r_con = r_client.get_connection().unwrap();

    for i in 1..100000 {
        vec_guy(&mut r_con);
        u8_guy(&mut r_con);
    }
}

fn vec_guy(c: &mut redis::Connection) {
    let gg: Vec<u8> = vec![0x20, 0x22, 0x33];
    let d1: redis::Value = c.zadd("XD", gg, 100).unwrap();
}

fn u8_guy(c: &mut redis::Connection) {
    let wp: [u8; 3] = [0x20, 0x22, 0x33];
    let d2: redis::Value = c.zadd("XD", &wp, 100).unwrap();
}
