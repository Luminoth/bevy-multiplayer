use axum::{
    debug_handler,
    extract::{Query, State},
    Json,
};
use axum_extra::TypedHeader;
use headers::authorization::{Authorization, Bearer};
use serde::Deserialize;
use tracing::{info, warn};
use uuid::Uuid;

use common::{gameclient::*, user::User};
use internal::axum::AppError;

use crate::{gameservers, state::AppState};

#[derive(Debug, Deserialize)]
pub struct FindServerParamsV1 {}

#[debug_handler]
pub async fn get_find_server_v1(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    Query(_params): Query<FindServerParamsV1>,
) -> Result<Json<FindServerResponseV1>, AppError> {
    let user = User::read_from_token(bearer.token()).await?;

    info!("finding game server for {} ...", user.user_id);

    let mut conn = app_state.redis_connection_pool.get_owned().await?;

    // TODO: check for reconnect

    // not reconnect, check for backfill
    if let Some(server_info) =
        gameservers::reserve_backfill_slot(&mut conn, &app_state, user.user_id).await?
    {
        return Ok(Json(FindServerResponseV1 {
            address: server_info.v4addrs[0].clone(),
            port: server_info.port,
        }));
    }

    info!("no backfill servers available, allocating session");

    let game_session_id = Uuid::new_v4();

    if let Some(server_info) =
        gameservers::allocate_game_server(&mut conn, &app_state, user.user_id, game_session_id)
            .await?
    {
        return Ok(Json(FindServerResponseV1 {
            address: server_info.v4addrs[0].clone(),
            port: server_info.port,
        }));
    }

    warn!("no placement servers available!");

    Ok(Json(FindServerResponseV1::default()))
}
