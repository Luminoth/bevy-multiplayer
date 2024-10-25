use bevy::prelude::*;
use bevy_mod_reqwest::*;

use crate::server::{GameServerInfo, GameSessionInfo};

pub fn heartbeat<'a>(
    client: &'a mut BevyReqwest,
    server_info: &GameServerInfo,
    session_info: Option<&GameSessionInfo>,
) -> BevyReqwestBuilder<'a> {
    debug!("heartbeat");

    let url = "http://localhost:8080/gameserver/heartbeat/v1";

    let req = client
        .post(url)
        .json(&common::gameserver::GameServerInfo {
            server_id: server_info.server_id,
            game_session_id: session_info.map(|session_info| session_info.session_id),
            player_session_ids: session_info
                .map(|session_info| session_info.player_session_ids.clone()),
            pending_player_ids: session_info
                .map(|session_info| session_info.pending_player_ids.clone()),
        })
        .build()
        .unwrap();

    client.send(req)
}
