use bb8_redis::redis::AsyncCommands;
use uuid::Uuid;

use internal::{
    gameserver::{get_gamesession_key, GAMESESSIONS_BACKFILL_SET},
    redis::RedisPooledConnection,
};

use crate::models;

pub async fn read_game_session_info(
    conn: &mut RedisPooledConnection,
    game_session_id: Uuid,
) -> anyhow::Result<Option<models::gameserver::GameSessionInfo>> {
    let game_session_info: Option<String> = conn.get(get_gamesession_key(game_session_id)).await?;
    if let Some(game_session_info) = game_session_info {
        return Ok(Some(serde_json::from_str(&game_session_info)?));
    }
    Ok(None)
}

pub async fn get_backfill_game_sessions(
    conn: &mut RedisPooledConnection,
) -> anyhow::Result<Vec<(String, u64)>> {
    Ok(conn.hgetall(GAMESESSIONS_BACKFILL_SET).await?)
}
