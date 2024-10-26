use axum::{debug_handler, extract::State, Json};
use bb8_redis::redis::{self};

use common::gameserver::*;

use crate::{error::AppError, models, state::AppState};

#[debug_handler]
pub async fn post_heartbeat_v1(
    State(app_state): State<AppState>,
    Json(request): Json<PostHeartbeatRequestV1>,
) -> Result<Json<PostHeartbeatResponseV1>, AppError> {
    let mut conn = app_state.redis_connection_pool.get().await?;

    let ttl = 60;
    let now = chrono::Utc::now().timestamp() as u64;
    let expiry = now - ttl;

    let mut pipeline = redis::pipe();

    let server_info_data = models::gameserver::GameServerInfo::from(request.server_info.clone());
    let value = serde_json::to_string(&server_info_data)?;
    pipeline.set_ex(server_info_data.get_key(), value, ttl);
    pipeline.zadd(
        "gameservers.index",
        server_info_data.server_id.to_string(),
        now,
    );
    pipeline.zrembyscore("gameservers.index", 0, expiry);

    if let Ok(session_info_data) =
        models::gameserver::GameSessionInfo::try_from(request.server_info)
    {
        let value = serde_json::to_string(&session_info_data)?;
        pipeline.set_ex(session_info_data.get_key(), value, ttl);
        pipeline.zadd(
            "gamesessions.index",
            session_info_data.game_session_id.to_string(),
            now,
        );
        pipeline.zrembyscore("gamesessions.index", 0, expiry);
    }

    let _: () = pipeline.query_async(&mut *conn).await?;

    Ok(Json(PostHeartbeatResponseV1 {}))
}
