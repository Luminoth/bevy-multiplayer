use bevy::prelude::*;
use bevy_mod_reqwest::*;
use uuid::Uuid;

use common::gameserver::*;

use crate::server::{ConnectionInfo, GameSessionInfo};

const API_HOST: &str = "http://localhost:8000";
const NOTIFS_HOST: &str = "ws://localhost:8001";

// TODO: have to use https://docs.rs/reqwest-websocket/latest/reqwest_websocket/ to get websocket support
pub fn subscribe<'a>(client: &'a mut BevyReqwest, server_id: Uuid) -> BevyReqwestBuilder<'a> {
    let url = format!("{}/notifs/v1", NOTIFS_HOST);

    let req: reqwest::Request = client
        .get(url)
        .header(
            http::header::AUTHORIZATION,
            // TODO: this is just hacky stuff until real auth is in
            format!("Bearer {}", server_id.to_string()),
        )
        .build()
        .unwrap();

    client.send(req)
}

pub fn heartbeat<'a>(
    client: &'a mut BevyReqwest,
    server_id: Uuid,
    connection_info: ConnectionInfo,
    state: GameServerState,
    orchestration: GameServerOrchestration,
    session_info: Option<&GameSessionInfo>,
) -> BevyReqwestBuilder<'a> {
    debug!("heartbeat");

    let url = format!("{}/gameserver/heartbeat/v1", API_HOST);

    let req = client
        .post(url)
        .json(&PostHeartbeatRequestV1 {
            server_info: GameServerInfo {
                server_id,
                addrs: connection_info.addrs,
                port: connection_info.port,
                state,
                orchestration,
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
