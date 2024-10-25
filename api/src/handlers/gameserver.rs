use axum::{debug_handler, extract::State, Json};
use bb8_redis::redis::{self};
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

    let mut pipeline = redis::pipe();

    let server_info_data = models::gameserver::GameServerInfo::from(server_info.clone());
    let value = serde_json::to_string(&server_info_data)?;
    let ttl = 60;
    pipeline.set_ex(server_info_data.get_key(), value, ttl);

    if let Ok(session_info_data) = models::gameserver::GameSessionInfo::try_from(server_info) {
        let value = serde_json::to_string(&session_info_data)?;
        let ttl = 60;
        pipeline.set_ex(session_info_data.get_key(), value, ttl);
    }

    let _: () = pipeline.query_async(&mut *conn).await?;

    Ok(Json(PostHeartbeatResponseV1 {}))
}
