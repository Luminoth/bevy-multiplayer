use bb8_redis::redis::AsyncCommands;
use tokio::time::{sleep, timeout, Duration};
use tracing::{info, warn};
use uuid::Uuid;

use common::user::UserId;
use internal::{
    gameserver::{get_gameserver_key, GAMESESSIONS_BACKFILL_SET, WAITING_GAMESERVERS_INDEX},
    notifs::AsNotification,
    redis::RedisPooledConnection,
};

use crate::{gamesessions, models, notifs, state::AppState};

const PLACEMENT_TIMEOUT: u64 = 60;

pub async fn read_gameserver_info(
    conn: &mut RedisPooledConnection,
    server_id: Uuid,
) -> anyhow::Result<Option<models::gameserver::GameServerInfo>> {
    let game_server_info: Option<String> = conn.get(get_gameserver_key(server_id)).await?;
    if let Some(game_server_info) = game_server_info {
        return Ok(Some(serde_json::from_str(&game_server_info)?));
    }
    Ok(None)
}

pub async fn reserve_backfill_slot(
    conn: &mut RedisPooledConnection,
) -> anyhow::Result<Option<models::gameserver::GameServerInfo>> {
    let backfill_sessions = gamesessions::get_backfill_game_sessions(conn).await?;
    if backfill_sessions.len() != 1 {
        warn!("no sessions available for backfill!");
        return Ok(None);
    }
    info!("{} sessions awaiting backfill", backfill_sessions.len());

    for (game_session_id, openslots) in backfill_sessions {
        if openslots < 1 {
            continue;
        }

        let game_session_id = Uuid::parse_str(&game_session_id)?;
        info!("checking backfill session {}", game_session_id);

        let game_session_info = gamesessions::read_game_session_info(conn, game_session_id).await?;
        if let Some(game_session_info) = game_session_info {
            info!("found backfill session {}", game_session_id);

            let server_info = read_gameserver_info(conn, game_session_info.server_id).await?;
            if let Some(server_info) = server_info {
                // TODO: not quite, we need to reserve the slot on the server
                // and then wait for it to be reserved, and THEN we can tell the client
                // and when we reserve we need to update "something" so we don't re-reserve
                // (and we should check for reserved slots here as well)

                return Ok(Some(server_info));
            } else {
                warn!("invalid backfill server {}", game_session_info.server_id);

                // TODO: cleanup
            }
        } else {
            warn!("invalid backfill session {}", game_session_id);

            let _: () = conn
                .hdel(GAMESESSIONS_BACKFILL_SET, game_session_id.to_string())
                .await?;
        }
    }

    Ok(None)
}

async fn wait_for_placement(
    conn: &mut RedisPooledConnection,
    server_id: Uuid,
    game_session_id: Uuid,
) -> anyhow::Result<Option<models::gameserver::GameServerInfo>> {
    info!("waiting for game session placement on {} ...", server_id);

    let key = get_gameserver_key(server_id);
    loop {
        // TODO: back off
        sleep(Duration::from_secs(1)).await;

        let server_info: String = conn.get(&key).await?;
        let server_info: models::gameserver::GameServerInfo = serde_json::from_str(&server_info)?;

        if let Some(current_session_id) = server_info.game_session_id {
            if current_session_id != game_session_id {
                warn!(
                    "placement session id mismatch, got {} expected {}!",
                    current_session_id, game_session_id
                );
                return Ok(None);
            }
        }

        if server_info.state == common::gameserver::GameServerState::InGame {
            return Ok(Some(server_info));
        }
    }
}

pub async fn allocate_game_server(
    conn: &mut RedisPooledConnection,
    app_state: &AppState,
    user_id: UserId,
    game_session_id: Uuid,
) -> anyhow::Result<Option<models::gameserver::GameServerInfo>> {
    let server_ids: Vec<(String, u64)> = conn.zpopmin(WAITING_GAMESERVERS_INDEX, 1).await?;
    if server_ids.len() != 1 {
        warn!("no game servers available for placement!");
        return Ok(None);
    }
    info!("{} servers available for placement", server_ids.len());

    let server_id = Uuid::parse_str(&server_ids[0].0)?;
    info!("found server for placement {}", server_id);

    let server_info = read_gameserver_info(conn, server_id).await?;
    if let Some(server_info) = server_info {
        if server_info.state != common::gameserver::GameServerState::WaitingForPlacement {
            // TODO: don't fail, try again until we can't find one
            warn!("server not waiting for placement!");
            return Ok(None);
        }

        notifs::notify_gameserver(
            &app_state,
            internal::notifs::PlacementRequestV1::new(game_session_id, vec![user_id])
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
            wait_for_placement(conn, server_id, game_session_id),
        )
        .await;
        if res.is_err() {
            warn!("placement timeout!");
            return Ok(None);
        }

        // TODO: if this comes back as None we should loop
        let server_info = res??.unwrap();

        return Ok(Some(server_info));
    } else {
        warn!("invalid placement server {}", server_id);
    }

    Ok(None)
}
