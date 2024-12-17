use bevy::prelude::*;
use bevy_mod_websocket::*;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use common::user::UserId;

const HOST: &str = "ws://localhost:8001";

fn on_success(trigger: Trigger<WebSocketConnectSuccessEvent>) {
    let evt = trigger.event();
    info!("subscribe success: {:?}", evt);
}

fn on_error(trigger: Trigger<WebSocketErrorEvent>) {
    let evt = trigger.event();
    // TODO: temp panic until we have retry
    //warn!("notifs error: {:?}", evt);
    panic!("notifs error: {:?}", evt);
}

fn on_disconnect(trigger: Trigger<WebSocketDisconnectEvent>) {
    let evt = trigger.event();
    // TODO: temp panic until we have reconnect
    //warn!("notifs disconnect: {:?}", evt);
    panic!("notifs disconnect: {:?}", evt);
}

fn on_message(trigger: Trigger<WebSocketMessageEvent>) {
    let evt = trigger.event();

    match &evt.message {
        Message::Text(value) => {
            info!("received notif from {}: {:?}", evt.uri, value);

            // TODO: error handling
            //let notif = serde_json::from_str::<notifs::Notification>(value).unwrap();
            //match notif.r#type {}
        }
        _ => {
            warn!("unexpected notif from {}: {:?}", evt.uri, evt.message);
        }
    }
}

pub fn subscribe<'a>(client: &'a mut WebSocketClient, user_id: UserId) -> WebSocketBuilder<'a> {
    // TODO: get rid of the need to call into_client_request so we can drop the tungstenite dependency
    let mut notifs_request = format!("{}/gameclient/notifs/v1", HOST)
        .into_client_request()
        .unwrap();
    let headers = notifs_request.headers_mut();
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {}", user_id).parse().unwrap(),
    );

    client
        .connect(notifs_request)
        .on_success(on_success)
        .on_error(on_error)
        .on_disconnect(on_disconnect)
        .on_message(on_message)
}
