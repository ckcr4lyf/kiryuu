use tokio_postgres::{NoTls, Error};

#[tokio::main(flavor = "current_thread")] // By default, tokio_postgres uses the tokio crate as its runtime.
async fn main() -> Result<(), Error> {
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

    // Now we can execute a simple statement that just returns its parameter.
    let rows = client
        .query("SELECT * FROM torrents", &[])
        .await?;

    // And then check that we got back the same string we sent over.
    let value: i64 = rows[0].get(1);
    println!("We got this: {}", value);

    // assert_eq!(value, "hello world");

    let n: i64 = 6969;
    let rows2 = client
        .query("INSERT INTO torrents VALUES($1, $2) ON CONFLICT (infohash) DO UPDATE SET last_announce = EXCLUDED.last_announce;", &[&"DAMN DANIEL", &n])
        .await?;

    println!("We got this: {:?}", rows2);

    Ok(())
}