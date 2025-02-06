use bevy::prelude::*;
use bevy_mod_reqwest::*;
use uuid::Uuid;

use common::{check_reqwest_error, gameserver};
use game_common::{
    network::ConnectionInfo,
    server::{ActivePlayer, GameSessionInfo, PendingPlayer},
};

const HOST: &str = "http://localhost:8000";

#[allow(clippy::too_many_arguments)]
pub fn heartbeat<'a>(
    client: &'a mut BevyReqwest,
    server_id: Uuid,
    connection_info: ConnectionInfo,
    state: gameserver::GameServerState,
    orchestration: gameserver::GameServerOrchestration,
    session_info: Option<&GameSessionInfo>,
    pending_players: impl Iterator<Item = &'a PendingPlayer>,
    active_players: impl Iterator<Item = &'a ActivePlayer>,
) -> anyhow::Result<BevyReqwestBuilder<'a>> {
    debug!("heartbeat");

    let url = format!("{}/gameserver/heartbeat/v1", HOST);

    let req = client
        .post(url)
        // TODO: should be auth JWT token
        .bearer_auth(server_id.to_string())
        .json(&gameserver::PostHeartbeatRequestV1 {
            server_info: gameserver::GameServerInfo {
                v4addrs: connection_info.v4addrs.iter().cloned().collect(),
                v6addrs: connection_info.v6addrs.iter().cloned().collect(),
                port: connection_info.port,
                state,
                orchestration,
                game_session_info: session_info.map(|session_info| gameserver::GameSessionInfo {
                    max_players: session_info.max_players,
                    game_session_id: session_info.session_id,
                    active_player_ids: active_players
                        .map(|active_player| active_player.user_id)
                        .collect(),
                    pending_player_ids: pending_players
                        .map(|pending_player| pending_player.user_id)
                        .collect(),
                }),
            },
        })
        .build()?;

    if let Some(session_info) = session_info {
        debug!("session_info: {:?}", session_info);
    }

    Ok(client
        .send(req)
        .on_response(|trigger: Trigger<ReqwestResponseEvent>| {
            check_reqwest_error(trigger.event());
        })
        .on_error(|trigger: Trigger<ReqwestErrorEvent>| {
            let e = &trigger.event().0;
            error!("heartbeat error: {:?}", e);
        }))
}
