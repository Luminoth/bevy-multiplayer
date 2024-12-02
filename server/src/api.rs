use bevy::prelude::*;
use bevy_mod_reqwest::*;
use uuid::Uuid;

use common::gameserver::*;

use crate::server::{ConnectionInfo, GameSessionInfo};

const HOST: &str = "http://localhost:8000";

pub fn heartbeat<'a>(
    client: &'a mut BevyReqwest,
    server_id: Uuid,
    connection_info: ConnectionInfo,
    state: GameServerState,
    orchestration: GameServerOrchestration,
    session_info: Option<&GameSessionInfo>,
) -> BevyReqwestBuilder<'a> {
    debug!("heartbeat");

    let url = format!("{}/gameserver/heartbeat/v1", HOST);

    let req = client
        .post(url)
        // TODO: should be auth JWT token
        .bearer_auth(server_id.to_string())
        .json(&PostHeartbeatRequestV1 {
            server_info: GameServerInfo {
                addrs: connection_info.addrs,
                port: connection_info.port,
                state,
                orchestration,
                max_players: session_info
                    .map(|session_info| session_info.max_players)
                    .unwrap_or_default(),
                game_session_id: session_info.map(|session_info| session_info.session_id),
                active_player_ids: session_info
                    .map(|session_info| session_info.active_player_ids.iter().copied().collect()),
                pending_player_ids: session_info
                    .map(|session_info| session_info.pending_player_ids.iter().copied().collect()),
            },
        })
        .build()
        .unwrap();

    client.send(req)
}
