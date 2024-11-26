use bevy::prelude::*;
use bevy_mod_reqwest::*;
use bevy_mod_websocket::*;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use uuid::Uuid;

use common::gameserver::*;

use crate::server::{ConnectionInfo, GameSessionInfo};

const API_HOST: &str = "http://localhost:8000";
const NOTIFS_HOST: &str = "ws://localhost:8001";

pub fn subscribe<'a>(client: &'a mut WebSocketClient, server_id: Uuid) -> WebSocketBuilder<'a> {
    let mut notifs_request = format!("{}/notifs/v1", NOTIFS_HOST)
        .into_client_request()
        .unwrap();
    let headers = notifs_request.headers_mut();
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {}", server_id).parse().unwrap(),
    );

    client.connect(notifs_request)
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
