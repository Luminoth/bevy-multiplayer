use axum::{
    debug_handler,
    extract::{Query, State},
    Json,
};
use axum_extra::TypedHeader;
use bb8_redis::redis::AsyncCommands;
use headers::authorization::{Authorization, Bearer};
use serde::Deserialize;
use tracing::{info, warn};

use common::{gameclient::*, user::User};
use internal::{axum::AppError, notifs::AsNotification};

use crate::{models, notifs, state::AppState};

#[derive(Debug, Deserialize)]
pub struct FindServerParamsV1 {}

#[debug_handler]
pub async fn get_find_server_v1(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    Query(_params): Query<FindServerParamsV1>,
) -> Result<Json<FindServerResponseV1>, AppError> {
    let user = User::read_from_token(bearer.token()).await?;

    info!("finding server for {} ...", user.user_id);

    let mut conn = app_state.redis_connection_pool.get().await?;

    // TODO: this isn't the best way to do this
    // we have all kinds of race conditions going on here

    let server_ids: Vec<(String, u64)> = conn.zpopmin("gameservers:waiting.index", 1).await?;
    if server_ids.len() != 1 {
        warn!("no servers available for placement!");
        return Ok(Json(FindServerResponseV1::default()));
    }

    let server_id = server_ids[0].0.clone();
    info!("found server {}", server_id);

    let server_info: String = conn.get(format!("gameserver:{}", server_id)).await?;
    let server_info: models::gameserver::GameServerInfo = serde_json::from_str(&server_info)?;

    notifs::notify_gameserver(
        &app_state,
        internal::notifs::PlacementRequestV1::default().as_notification(server_id)?,
        Some(30),
    )
    .await?;

    // TODO: wait for the server (with a timeout)

    Ok(Json(FindServerResponseV1 {
        address: "127.0.0.1".to_string(), //server_info.addrs[0].clone(),
        port: server_info.port,
    }))
}
