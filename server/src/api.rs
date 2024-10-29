use bevy::prelude::*;
use bevy_mod_reqwest::*;
use uuid::Uuid;

use common::gameserver::*;

use crate::server::GameSessionInfo;

pub fn heartbeat<'a>(
    client: &'a mut BevyReqwest,
    server_id: Uuid,
    state: GameServerState,
    // TODO: connection info
    session_info: Option<&GameSessionInfo>,
) -> BevyReqwestBuilder<'a> {
    debug!("heartbeat");

    let url = "http://localhost:8080/gameserver/heartbeat/v1";

    let req = client
        .post(url)
        .json(&PostHeartbeatRequestV1 {
            server_info: GameServerInfo {
                server_id,
                state,
                // TODO: connection info
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
