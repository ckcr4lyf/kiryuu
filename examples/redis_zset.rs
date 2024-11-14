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
        redis::Value::Set(v1) => v1,
        _ => vec![]
    };

    let mut seeders = [0_u8; 50 * 6]; // We will store max 50 guys
    let mut pos = 0;

    for element in &zrange_result {
        if let redis::Value::BulkString(xd) = element {
            for i in 0..6 {
                seeders[pos + i] = xd[i];
            }
            pos += 6;
        }

        if pos >= 300 {
            break;
        }
    }

    println!("Manual seeders are {:?}", seeders);


    let dummy: Vec<u8> = vec![0u8; 1];
    let mut seeders_v2: [&Vec<u8>; 50] = [&dummy; 50];

    pos = 0;

    for element in &zrange_result {
        if let redis::Value::BulkString(xd) = element {
            seeders_v2[pos] = xd;
            pos += 1;
        }
    }

    println!("Manual seeders 2 are {:?}", seeders_v2);



    // let mut body: [u8; 10] = [0; 10];
    // if let redis::Value::Data(xd) = zrange_result.get(0).unwrap() {
    //     body[0] = xd[0];
    // }

    // // I Know I have a Vec<redis::Value> now
    // println!("zrange_result is {:?}", zrange_result);
    // println!("body is {:?}", body);

    let og: Vec<Vec<u8>> = r_client.zrangebyscore("MYKEY", "0", "101").unwrap();
    println!("og is {:?}", og)
}

/* Manual testing:
FLUSHDB
ZADD MYKEY 10 ABCDEF
ZADD MYKEY 10 XXXXXX
*/
