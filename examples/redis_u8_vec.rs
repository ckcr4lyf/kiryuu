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

struct InfoHash([u8; 40]);

struct RawVal<const T: usize>([u8; T]);

impl redis::ToRedisArgs for InfoHash {
    fn write_redis_args<W>(&self, out: &mut W) where W: ?Sized + redis::RedisWrite {
        out.write_arg(&self.0)
    }
}

impl<const T: usize> redis::ToRedisArgs for RawVal<T> {
    fn write_redis_args<W>(&self, out: &mut W) where W: ?Sized + redis::RedisWrite {
        out.write_arg(&self.0)
    }
}

fn u8_guy(c: &mut redis::Connection) {
    // let wp: [u8; 3] = [0x20, 0x22, 0x33];
    let wp: [u8; 40] = [0; 40];
    let WP = RawVal(wp);
    let d2: redis::Value = c.zadd("XD", WP, 100).unwrap();
}
