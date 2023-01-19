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

    let mut rClient = redis::Client::open("redis://127.0.0.1:6379").unwrap();

    // let theZscore: f64 = rClient.zscore("thezset", "KEY1").unwrap();
    let theZscore: Exists = rClient.zscore("thezset", "KEY2").unwrap();


    println!("the zscore is {:?}", theZscore)
}