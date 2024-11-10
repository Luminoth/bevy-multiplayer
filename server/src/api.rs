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
    session_info: Option<&GameSessionInfo>,
) -> BevyReqwestBuilder<'a> {
    debug!("heartbeat");

    let url = format!("{}/gameserver/heartbeat/v1", HOST);

    let req = client
        .post(url)
        .json(&PostHeartbeatRequestV1 {
            server_info: GameServerInfo {
                server_id,
                addrs: connection_info.addrs,
                port: connection_info.port,
                state,
                max_players: session_info
                    .map(|session_info| session_info.max_players)
                    .unwrap_or_default(),
                game_session_id: session_info.map(|session_info| session_info.session_id),
                player_session_ids: session_info
                    .map(|session_info| session_info.player_session_ids.clone()),
                pending_player_ids: session_info
                    .map(|session_info| session_info.pending_player_ids.clone()),
            },
        })
        .build()
        .unwrap();

    client.send(req)
}
