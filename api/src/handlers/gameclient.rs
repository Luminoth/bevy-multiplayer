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
    gameserver::{
        get_gameserver_key, get_gamesession_key, GAMESESSIONS_BACKFILL_SET,
        WAITING_GAMESERVERS_INDEX,
    },
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

    // first backfill
    let backfill_sessions: Vec<(String, u64)> = conn.hgetall(GAMESESSIONS_BACKFILL_SET).await?;
    for (game_session_id, openslots) in backfill_sessions {
        if openslots > 0 {
            let game_session_id = Uuid::parse_str(&game_session_id)?;
            info!("checking backfill session {}", game_session_id);

            let game_session_info: Option<String> =
                conn.get(get_gamesession_key(game_session_id)).await?;
            if let Some(game_session_info) = game_session_info {
                info!("found backfill session {}", game_session_id);

                let game_session_info: models::gameserver::GameSessionInfo =
                    serde_json::from_str(&game_session_info)?;

                let server_info: Option<String> = conn
                    .get(get_gameserver_key(game_session_info.server_id))
                    .await?;
                if let Some(server_info) = server_info {
                    let server_info: models::gameserver::GameServerInfo =
                        serde_json::from_str(&server_info)?;

                    // TODO: not quite, we need to reserve the slot on the server
                    // and then wait for it to be reserved, and THEN we can tell the client
                    // and when we reserve we need to update "something" so we don't re-reserve
                    // (and we should check for reserved slots here as well)

                    return Ok(Json(FindServerResponseV1 {
                        address: server_info.addrs[0].clone(),
                        port: server_info.port,
                    }));
                } else {
                    info!("invalid backfill server {}", game_session_info.server_id);

                    // TODO: cleanup
                }
            } else {
                info!("invalid backfill session {}", game_session_id);

                let _: () = conn
                    .hdel(GAMESESSIONS_BACKFILL_SET, game_session_id.to_string())
                    .await?;
            }
        }
    }

    warn!("no backfill servers available!");

    let server_ids: Vec<(String, u64)> = conn.zpopmin(WAITING_GAMESERVERS_INDEX, 1).await?;
    if server_ids.len() != 1 {
        warn!("no game servers available for placement!");
        return Ok(Json(FindServerResponseV1::default()));
    }

    let server_id = Uuid::parse_str(&server_ids[0].0)?;
    info!("found server for placement {}", server_id);

    let server_info: Option<String> = conn.get(get_gameserver_key(server_id)).await?;
    if let Some(server_info) = server_info {
        let server_info: models::gameserver::GameServerInfo = serde_json::from_str(&server_info)?;

        // TODO: if the server is now running and has room then we should return it here
        if server_info.state != common::gameserver::GameServerState::WaitingForPlacement {
            // TODO: don't fail, try again until we can't find one
            warn!("server not waiting for placement!");
            return Ok(Json(FindServerResponseV1::default()));
        }

        notifs::notify_gameserver(
            &app_state,
            internal::notifs::PlacementRequestV1::new(vec![user.user_id])
                .as_notification(server_id)?,
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

        return Ok(Json(FindServerResponseV1 {
            address: server_info.addrs[0].clone(),
            port: server_info.port,
        }));
    } else {
        info!("invalid placement server {}", server_id);
    }

    // TODO: should be error?
    Ok(Json(FindServerResponseV1::default()))
}
