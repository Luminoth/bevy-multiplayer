use redis::{aio::ConnectionManager, AsyncCommands};
use tracing::info;

pub type RedisConnection = ConnectionManager;

async fn ping(conn: &mut RedisConnection) -> anyhow::Result<()> {
    let pong: String = conn.ping().await?;
    if pong != "PONG" {
        anyhow::bail!("failed to connect to redis");
    }

    Ok(())
}

pub async fn connect(address: impl AsRef<str>) -> anyhow::Result<RedisConnection> {
    let address = address.as_ref();

    info!("connecting redis at {} ...", address);

    let client = redis::Client::open(address)?;
    let mut conn = ConnectionManager::new(client).await?;

    ping(&mut conn).await?;

    Ok(conn)
}
