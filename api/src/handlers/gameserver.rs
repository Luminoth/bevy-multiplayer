use axum::{debug_handler, extract::State, Json};
use serde::Serialize;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Serialize)]
pub struct PostHeartbeatResponseV1 {}

#[debug_handler]
pub async fn post_heartbeat_v1(
    State(_app_state): State<AppState>,
) -> Result<Json<PostHeartbeatResponseV1>, AppError> {
    Ok(Json(PostHeartbeatResponseV1 {}))
}
