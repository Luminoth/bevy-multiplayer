use bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;
use redis::IntoConnectionInfo;
use tracing::info;

pub type RedisConnectionPool = Pool<RedisConnectionManager>;
pub type RedisPooledConnection = PooledConnection<'static, RedisConnectionManager>;

async fn ping(conn: &mut RedisPooledConnection) -> anyhow::Result<()> {
    let result = conn.send_packed_command(&redis::cmd("PING")).await?;
    if result != redis::Value::SimpleString("PONG".into()) {
        anyhow::bail!("failed to connect to redis");
    }

    Ok(())
}

pub async fn connect<T>(info: T) -> anyhow::Result<RedisConnectionPool>
where
    T: IntoConnectionInfo,
{
    info!("connecting redis ...");

    let manager = RedisConnectionManager::new(info)?;
    let pool = Pool::builder().build(manager).await?;

    let mut conn = pool.get_owned().await?;
    ping(&mut conn).await?;

    Ok(pool)
}
