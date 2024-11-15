use axum::{
    debug_handler,
    extract::{Query, State},
    Json,
};
use axum_extra::TypedHeader;
use bb8_redis::redis::AsyncCommands;
use headers::authorization::{Authorization, Bearer};
use serde::Deserialize;
use tracing::warn;

use common::{gameclient::*, user::User};
use internal::axum::AppError;

use crate::{models, state::AppState};

#[derive(Debug, Deserialize)]
pub struct FindServerParamsV1 {}

#[debug_handler]
pub async fn get_find_server_v1(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    Query(_params): Query<FindServerParamsV1>,
) -> Result<Json<FindServerResponseV1>, AppError> {
    let _user = User::read_from_token(bearer.token()).await?;

    let mut conn = app_state.redis_connection_pool.get().await?;

    // TODO: this isn't the best way to do this
    // we have all kinds of race conditions going on here

    let server_ids: Vec<(String, u64)> = conn.zpopmin("gameservers:waiting.index", 1).await?;
    if server_ids.len() != 1 {
        warn!("no servers available for placement!");
        return Ok(Json(FindServerResponseV1::default()));
    }

    let server_info: String = conn.get(format!("gameserver:{}", server_ids[0].0)).await?;
    let server_info: models::gameserver::GameServerInfo = serde_json::from_str(&server_info)?;

    // TODO: notify the server

    Ok(Json(FindServerResponseV1 {
        address: "127.0.0.1".to_string(), //server_info.addrs[0].clone(),
        port: server_info.port,
    }))
}
