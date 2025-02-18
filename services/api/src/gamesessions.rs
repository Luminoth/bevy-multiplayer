use redis::{AsyncCommands, Pipeline};
use uuid::Uuid;

use internal::{
    gameserver::{get_gamesession_key, GAMESESSIONS_BACKFILL_SET, GAMESESSIONS_INDEX},
    redis::RedisConnection,
};

use crate::models;

const SESSION_INFO_TTL: u64 = 60;

pub async fn read_game_session_info(
    conn: &mut RedisConnection,
    game_session_id: Uuid,
) -> anyhow::Result<Option<models::gamesession::GameSessionInfo>> {
    let game_session_info: Option<String> = conn.get(get_gamesession_key(game_session_id)).await?;
    if let Some(game_session_info) = game_session_info {
        return Ok(Some(serde_json::from_str(&game_session_info)?));
    }
    Ok(None)
}

pub async fn update_game_session(
    pipeline: &mut Pipeline,
    game_session_info: &models::gamesession::GameSessionInfo,
) -> anyhow::Result<()> {
    let value = serde_json::to_string(&game_session_info)?;

    let now = chrono::Utc::now().timestamp() as u64;
    let expiry = now - SESSION_INFO_TTL;

    // save the session info
    pipeline.set_ex(
        get_gamesession_key(game_session_info.game_session_id),
        value,
        SESSION_INFO_TTL,
    );

    // update the session index
    pipeline.zadd(
        GAMESESSIONS_INDEX,
        game_session_info.game_session_id.to_string(),
        now,
    );
    pipeline.zrembyscore(GAMESESSIONS_INDEX, 0, expiry);

    // update sessions that need backfill
    let openslots = game_session_info.player_slots_remaining();
    if openslots > 0 {
        pipeline.hset(
            GAMESESSIONS_BACKFILL_SET,
            game_session_info.game_session_id.to_string(),
            openslots,
        );
    } else {
        pipeline.hdel(
            GAMESESSIONS_BACKFILL_SET,
            game_session_info.game_session_id.to_string(),
        );
    }

    Ok(())
}

pub async fn get_backfill_game_sessions(
    conn: &mut RedisConnection,
) -> anyhow::Result<Vec<(String, u64)>> {
    Ok(conn.hgetall(GAMESESSIONS_BACKFILL_SET).await?)
}
