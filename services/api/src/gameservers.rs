use redis::{AsyncCommands, Pipeline};
use tokio::time::{sleep, timeout, Duration};
use tracing::{info, warn};
use uuid::Uuid;

use common::{gameserver::*, user::UserId};
use internal::{
    gameserver::{
        get_gameserver_key, get_gamesession_key, GAMESERVERS_INDEX, GAMESESSIONS_BACKFILL_SET,
        WAITING_GAMESERVERS_INDEX,
    },
    notifs::AsNotification,
    redis::RedisConnection,
};

use crate::{gamesessions, models, notifs, state::AppState};

const PLACEMENT_TIMEOUT: Duration = Duration::from_secs(30);
const RESERVATION_TIMEOUT: Duration = Duration::from_secs(5);
const SERVER_INFO_TTL: u64 = 10;

pub async fn read_gameserver_info(
    conn: &mut RedisConnection,
    server_id: Uuid,
) -> anyhow::Result<Option<models::gameserver::GameServerInfo>> {
    let game_server_info: Option<String> = conn.get(get_gameserver_key(server_id)).await?;
    if let Some(game_server_info) = game_server_info {
        return Ok(Some(serde_json::from_str(&game_server_info)?));
    }
    Ok(None)
}

pub async fn update_gameserver(
    pipeline: &mut Pipeline,
    gameserver_info: &models::gameserver::GameServerInfo,
) -> anyhow::Result<()> {
    let value = serde_json::to_string(&gameserver_info)?;

    let now = chrono::Utc::now().timestamp() as u64;
    let expiry = now - SERVER_INFO_TTL;

    // save the server info
    pipeline.set_ex(
        get_gameserver_key(gameserver_info.server_id),
        value,
        SERVER_INFO_TTL,
    );

    // update the server index
    pipeline.zadd(
        GAMESERVERS_INDEX,
        gameserver_info.server_id.to_string(),
        now,
    );
    pipeline.zrembyscore(GAMESERVERS_INDEX, 0, expiry);

    // update servers waiting for placement
    if gameserver_info.state == GameServerState::WaitingForPlacement {
        pipeline.zadd(
            WAITING_GAMESERVERS_INDEX,
            gameserver_info.server_id.to_string(),
            now,
        );
        pipeline.zrembyscore(WAITING_GAMESERVERS_INDEX, 0, expiry);
    }

    Ok(())
}

async fn wait_for_reservation(
    conn: &mut RedisConnection,
    game_session_id: Uuid,
    user_id: UserId,
) -> anyhow::Result<()> {
    info!("waiting for reservation on {} ...", game_session_id);

    let key = get_gamesession_key(game_session_id);
    loop {
        // TODO: back off
        sleep(Duration::from_secs(1)).await;

        let game_session_info: String = conn.get(&key).await?;
        let game_session_info: models::gamesession::GameSessionInfo =
            serde_json::from_str(&game_session_info)?;

        if game_session_info.pending_player_ids.contains(&user_id) {
            return Ok(());
        }
    }
}

pub async fn reserve_backfill_slot(
    app_state: &mut AppState,
    user_id: UserId,
) -> anyhow::Result<Option<models::gameserver::GameServerInfo>> {
    let backfill_sessions =
        gamesessions::get_backfill_game_sessions(&mut app_state.redis_connection).await?;
    if backfill_sessions.is_empty() {
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

        let game_session_info =
            gamesessions::read_game_session_info(&mut app_state.redis_connection, game_session_id)
                .await?;
        if let Some(game_session_info) = game_session_info {
            info!("found backfill session {}", game_session_id);

            let server_info =
                read_gameserver_info(&mut app_state.redis_connection, game_session_info.server_id)
                    .await?;
            if let Some(server_info) = server_info {
                notifs::notify_gameserver(
                    app_state,
                    internal::notifs::ReservationRequestV1::new(game_session_id, vec![user_id])
                        .as_notification(server_info.server_id)?,
                    Some(RESERVATION_TIMEOUT),
                )
                .await?;

                // TODO: polling for this kind of sucks,
                // instead what if we notified clients that are part of the session
                // when the game server heartbeat happens ?
                // that should have a faster turnaround?
                // clients would need a mailbox poll tho (probably will anyway)
                // or a "send messages on notifs connect" piece

                let res = timeout(
                    RESERVATION_TIMEOUT,
                    wait_for_reservation(&mut app_state.redis_connection, game_session_id, user_id),
                )
                .await;
                if res.is_err() {
                    warn!("reservation timeout!");
                    return Ok(None);
                }

                return Ok(Some(server_info));
            } else {
                warn!("invalid backfill server {}", game_session_info.server_id);

                // TODO: cleanup
            }
        } else {
            warn!("invalid backfill session {}", game_session_id);

            let _: () = app_state
                .redis_connection
                .hdel(GAMESESSIONS_BACKFILL_SET, game_session_id.to_string())
                .await?;
        }
    }

    Ok(None)
}

async fn wait_for_placement(
    conn: &mut RedisConnection,
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
    app_state: &mut AppState,
    user_id: UserId,
    game_session_id: Uuid,
) -> anyhow::Result<Option<models::gameserver::GameServerInfo>> {
    let server_ids: Vec<(String, u64)> = app_state
        .redis_connection
        .zpopmin(WAITING_GAMESERVERS_INDEX, 1)
        .await?;
    if server_ids.len() != 1 {
        warn!("no game servers available for placement!");
        return Ok(None);
    }
    info!("{} servers available for placement", server_ids.len());

    let server_id = Uuid::parse_str(&server_ids[0].0)?;
    info!("found server for placement {}", server_id);

    let server_info = read_gameserver_info(&mut app_state.redis_connection, server_id).await?;
    if let Some(server_info) = server_info {
        if server_info.state != common::gameserver::GameServerState::WaitingForPlacement {
            // TODO: don't fail, try again until we can't find one
            warn!("server not waiting for placement!");
            return Ok(None);
        }

        notifs::notify_gameserver(
            app_state,
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
            PLACEMENT_TIMEOUT,
            wait_for_placement(&mut app_state.redis_connection, server_id, game_session_id),
        )
        .await;
        if res.is_err() {
            warn!("placement timeout!");
            return Ok(None);
        }

        info!("session {} placed on {}", game_session_id, server_id);

        // TODO: if this comes back as None we should loop
        let server_info = res??.unwrap();

        return Ok(Some(server_info));
    } else {
        warn!("invalid placement server {}", server_id);
    }

    Ok(None)
}
