use bevy::prelude::*;
use bevy_mod_reqwest::*;
use uuid::Uuid;

use common::{check_reqwest_error, gameserver};

use crate::server::{ConnectionInfo, GameSessionInfo};

const HOST: &str = "http://localhost:8000";

pub fn heartbeat<'a>(
    client: &'a mut BevyReqwest,
    server_id: Uuid,
    connection_info: ConnectionInfo,
    state: gameserver::GameServerState,
    orchestration: gameserver::GameServerOrchestration,
    session_info: Option<&GameSessionInfo>,
) -> anyhow::Result<BevyReqwestBuilder<'a>> {
    debug!("heartbeat");

    let url = format!("{}/gameserver/heartbeat/v1", HOST);

    let req = client
        .post(url)
        // TODO: should be auth JWT token
        .bearer_auth(server_id.to_string())
        .json(&gameserver::PostHeartbeatRequestV1 {
            server_info: gameserver::GameServerInfo {
                addrs: connection_info.addrs,
                port: connection_info.port,
                state,
                orchestration,
                game_session_info: session_info.map(|session_info| gameserver::GameSessionInfo {
                    max_players: session_info.max_players,
                    game_session_id: session_info.session_id,
                    active_player_ids: session_info.active_player_ids.iter().copied().collect(),
                    pending_player_ids: session_info.pending_player_ids.iter().copied().collect(),
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
