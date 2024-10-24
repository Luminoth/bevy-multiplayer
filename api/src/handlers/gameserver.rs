use axum::{debug_handler, extract::State, Json};
use redis::AsyncCommands;
use serde::Serialize;

use common::gameserver::*;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Serialize)]
pub struct PostHeartbeatResponseV1 {}

#[debug_handler]
pub async fn post_heartbeat_v1(
    State(app_state): State<AppState>,
    Json(info): Json<GameServerInfo>,
) -> Result<Json<PostHeartbeatResponseV1>, AppError> {
    let mut conn = app_state.redis_connection_pool.get().await?;

    let key = format!("gameserver:{}", info.server_id);
    let value = serde_json::to_string(&info)?;
    let ttl = 60;
    let _: () = conn.set_ex(key, value, ttl).await?;

    Ok(Json(PostHeartbeatResponseV1 {}))
}
