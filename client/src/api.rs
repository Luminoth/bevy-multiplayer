use bevy::prelude::*;
use bevy_mod_reqwest::*;
use bevy_mod_websocket::*;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use common::user::UserId;

const API_HOST: &str = "http://localhost:8000";
const NOTIFS_HOST: &str = "ws://localhost:8001";

pub fn subscribe<'a>(client: &'a mut WebSocketClient, user_id: UserId) -> WebSocketBuilder<'a> {
    // TODO: get rid of the need to call into_client_request so we can drop the tungstenite dependency
    let mut notifs_request = format!("{}/gameclient/notifs/v1", NOTIFS_HOST)
        .into_client_request()
        .unwrap();
    let headers = notifs_request.headers_mut();
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {}", user_id).parse().unwrap(),
    );

    client.connect(notifs_request)
}

pub fn find_server<'a>(
    client: &'a mut BevyReqwest,
    user_id: UserId,
) -> anyhow::Result<BevyReqwestBuilder<'a>> {
    info!("finding server ...");

    let url = format!("{}/gameclient/find_server/v1", API_HOST);

    let req = client
        .get(url)
        // TODO: should be auth JWT token
        .bearer_auth(user_id.to_string())
        .build()?;

    Ok(client.send(req))
}
