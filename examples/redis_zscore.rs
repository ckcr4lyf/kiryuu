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

    // We work with a custom enum which implements FromRedisValue
    let exist_highlevel = match r_client.zscore::<_, _, Exists>("thezset", "KEY2").unwrap() {
        Exists::No => false,
        Exists::Yes => true,
    };

    // We work with lower level value, since we only need to detect nil or non-nil. This can save us from type checking (I think?)
    let exist_lowlevel = match r_client.zscore::<_, _, redis::Value>("thezset", "KEY2").unwrap() {
        redis::Value::Nil => false,
        _ => true,
    };

    println!("exist_highlevel is {:?}, exist_lowlevel is {:?}", exist_highlevel, exist_lowlevel)
}