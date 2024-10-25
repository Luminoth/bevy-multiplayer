use bb8_redis::{
    bb8::{Pool, PooledConnection},
    redis::cmd,
    RedisConnectionManager,
};
use tracing::info;

pub type RedisConnectionPool = Pool<RedisConnectionManager>;
pub type RedisPooledConnection = PooledConnection<'static, RedisConnectionManager>;

async fn ping(conn: &mut RedisPooledConnection) -> anyhow::Result<()> {
    let pong: String = cmd("PING").query_async(&mut **conn).await?;
    if pong != "PONG" {
        anyhow::bail!("failed to connect to redis");
    }

    Ok(())
}

pub async fn connect(address: impl AsRef<str>) -> anyhow::Result<RedisConnectionPool> {
    let address = address.as_ref();

    info!("connecting redis at {} ...", address);

    let manager = RedisConnectionManager::new(address)?;
    let pool = Pool::builder().build(manager).await?;

    let mut conn = pool.get_owned().await?;
    ping(&mut conn).await?;

    Ok(pool)
}
