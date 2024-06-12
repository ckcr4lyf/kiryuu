use std::time::{SystemTime, UNIX_EPOCH};
use kiryuu::byte_functions::types::RawVal;
use log::{debug, info};
use tokio_postgres::{Error, NoTls};

/// cleanup job involves
/// -> Get all TORRENTS that are older than 31 minutes
/// -> Delete their {hash}_seeders, {hash}_leechers ZSET
/// -> Delete their {hash} HASH

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    env_logger::init();


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

    let rows = client.query("SELECT * FROM torrents WHERE last_announce < $1", &[&max_limit]).await?;

    for row in rows {
        let infohash: RawVal<40> = row.get(0);
        debug!("Going to handle row {}", String::from_utf8(infohash.0.to_vec()).expect("fuck"));

        // pipeline to delete keys from redis
    }

    // execute pipeline
    
    Ok(())
}
