use std::time::{SystemTime, UNIX_EPOCH};
use kiryuu::byte_functions::{self, types::RawVal};
use log::{debug, info};
use tokio_postgres::{Error, NoTls};

/// cleanup job involves
/// -> Get all TORRENTS that are older than 31 minutes
/// -> Delete their {hash}_seeders, {hash}_leechers ZSET
/// -> Delete their {hash} HASH

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    env_logger::init();
    info!("starting clean job");

    // Connect to redis
    let redis = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut redis_connection = redis.get_multiplexed_tokio_connection().await.unwrap();

    // Connect to the database.
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=password", NoTls).await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let time_now = SystemTime::now().duration_since(UNIX_EPOCH).expect("fucked up");
    let time_now_ms: i64 = i64::try_from(time_now.as_millis()).expect("fucc");
    let max_limit = time_now_ms - (1000 * 60 * 31);
    // let max_limit = time_now_ms;

    let rows = client.query("SELECT * FROM torrents WHERE last_announce < $1", &[&max_limit]).await?;
    info!("Got {} torrents to clean!", rows.len());

    for row in rows {
        let infohash: RawVal<40> = row.get(0);
        debug!("Going to handle row {}", String::from_utf8(infohash.0.to_vec()).expect("fuck"));

        // pipeline to delete keys from redis
        // basically if the TORRENT's last announce is more than 31 minutes ago, we can delete the
        // _seeders , _leechers & _cache keys
        let (skey, lkey, ckey) = byte_functions::make_redis_keys(&infohash);
        let cmd: bool = redis::cmd("DEL").arg(&skey).arg(&lkey).arg(&ckey).query_async(&mut redis_connection).await.expect("fucc");
        debug!("result of clean {:?}", cmd);
    }

    info!("Finished clean job");

    // execute pipeline
    
    Ok(())
}
