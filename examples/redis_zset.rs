use redis::{self, Commands};

#[derive(Debug)]
enum Exists {
    Yes,
    No,
}

impl redis::FromRedisValue for Exists {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Exists> {
        match *v {
            redis::Value::Nil => Ok(Exists::No),
            _ => Ok(Exists::Yes),
        }
    }
}

fn main(){
    let mut r_client = redis::Client::open("redis://127.0.0.1:6379").unwrap();

    // Work with low val
    let zrange_result = match r_client.zrangebyscore::<_, _, _, redis::Value>("MYKEY", "0", "101").unwrap() {
        redis::Value::Bulk(v1) => v1,
        _ => vec![]
    };

    let mut body: [u8; 10] = [0; 10];
    if let redis::Value::Data(xd) = zrange_result.get(0).unwrap() {
        body[0] = xd[0];
    }

    // I Know I have a Vec<redis::Value> now
    println!("zrange_result is {:?}", zrange_result);
    println!("body is {:?}", body);

    let og: Vec<Vec<u8>> = r_client.zrangebyscore("MYKEY", "0", "101").unwrap();
    println!("og is {:?}", og)
}