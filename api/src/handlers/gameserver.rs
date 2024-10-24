use axum::{debug_handler, extract::State, Json};
use redis::AsyncCommands;
use serde::Serialize;

use crate::{error::AppError, models, state::AppState};

#[derive(Debug, Serialize)]
pub struct PostHeartbeatResponseV1 {}

#[debug_handler]
pub async fn post_heartbeat_v1(
    State(app_state): State<AppState>,
    Json(server_info): Json<common::gameserver::GameServerInfo>,
) -> Result<Json<PostHeartbeatResponseV1>, AppError> {
    let mut conn = app_state.redis_connection_pool.get().await?;

    // TODO: pipeline this

    let server_info_data = models::gameserver::GameServerInfo::from(server_info.clone());
    let value = serde_json::to_string(&server_info_data)?;
    let ttl = 60;
    let _: () = conn.set_ex(server_info_data.get_key(), value, ttl).await?;

    if let Ok(session_info_data) = models::gameserver::GameSessionInfo::try_from(server_info) {
        let value = serde_json::to_string(&session_info_data)?;
        let ttl = 60;
        let _: () = conn.set_ex(session_info_data.get_key(), value, ttl).await?;
    }

    Ok(Json(PostHeartbeatResponseV1 {}))
}
