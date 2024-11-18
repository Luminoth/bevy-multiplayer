use axum::{
    debug_handler,
    extract::{Query, State},
    Json,
};
use axum_extra::TypedHeader;
use bb8_redis::redis::AsyncCommands;
use headers::authorization::{Authorization, Bearer};
use serde::Deserialize;
use tokio::time::{sleep, timeout, Duration};
use tracing::{info, warn};
use uuid::Uuid;

use common::{gameclient::*, user::User};
use internal::{
    axum::AppError,
    gameserver::{get_gameserver_key, WAITING_GAMESERVERS_INDEX},
    notifs::AsNotification,
    redis::RedisPooledConnection,
};

use crate::{models, notifs, state::AppState};

const PLACEMENT_TIMEOUT: u64 = 60;

async fn wait_for_placement(
    conn: &mut RedisPooledConnection,
    server_id: Uuid,
) -> anyhow::Result<models::gameserver::GameServerInfo> {
    info!("waiting for game session placement on {} ...", server_id);

    let key = get_gameserver_key(server_id);
    loop {
        // TODO: back off
        sleep(Duration::from_secs(1)).await;

        let server_info: String = conn.get(&key).await?;
        let server_info: models::gameserver::GameServerInfo = serde_json::from_str(&server_info)?;

        // TODO: should use a ready flag or something instead?
        if server_info.state == common::gameserver::GameServerState::InGame {
            return Ok(server_info);
        }
    }
}

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

    // TODO: look for not full servers before checking waiting servers

    let server_ids: Vec<(String, u64)> = conn.zpopmin(WAITING_GAMESERVERS_INDEX, 1).await?;
    if server_ids.len() != 1 {
        warn!("no game servers available for placement!");
        return Ok(Json(FindServerResponseV1::default()));
    }

    let server_id = Uuid::parse_str(&server_ids[0].0)?;
    info!("found server {}", server_id);

    let server_info: String = conn.get(get_gameserver_key(server_id)).await?;
    let server_info: models::gameserver::GameServerInfo = serde_json::from_str(&server_info)?;

    // TODO: if the server is now running and has room then we should return it here
    if server_info.state != common::gameserver::GameServerState::WaitingForPlacement {
        // TODO: don't fail, try again until we can't find one
        warn!("server not waiting for placement!");
        return Ok(Json(FindServerResponseV1::default()));
    }

    notifs::notify_gameserver(
        &app_state,
        internal::notifs::PlacementRequestV1::new(vec![user.user_id]).as_notification(server_id)?,
        Some(PLACEMENT_TIMEOUT),
    )
    .await?;

    // TODO: polling for this kind of sucks,
    // instead what if we notified clients that are part of the session
    // when the game server heartbeat happens ?
    // that should have a faster turnaround?
    // clients would need a mailbox poll tho (probably will anyway)
    // or a "send messages on notifs connect" piece

    let res = timeout(
        Duration::from_secs(PLACEMENT_TIMEOUT),
        wait_for_placement(&mut conn, server_id),
    )
    .await;
    if res.is_err() {
        warn!("placement timeout!");
        return Ok(Json(FindServerResponseV1::default()));
    }

    let server_info = res??;

    Ok(Json(FindServerResponseV1 {
        address: server_info.addrs[0].clone(),
        port: server_info.port,
    }))
}
