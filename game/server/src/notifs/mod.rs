mod placement;
mod reservation;

use bevy::{prelude::*, utils::Duration};
use bevy_mod_websocket::*;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use uuid::Uuid;

use internal::notifs;

use crate::{
    server::{GameSessionInfo, HeartbeatEvent},
    AppState,
};

const HOST: &str = "ws://localhost:8001";
const RETRY_INTERVAL: Duration = Duration::from_secs(10);

fn on_success(trigger: Trigger<WebSocketConnectSuccessEvent>) {
    let evt = trigger.event();
    info!("subscribe success: {:?}", evt);
}

fn on_error(trigger: Trigger<WebSocketErrorEvent>, mut ws_client: WebSocketClient) {
    let evt = trigger.event();
    warn!("notifs error: {:?}", evt.error);

    ws_client.retry(trigger.entity(), evt.request.clone(), RETRY_INTERVAL);
}

fn on_disconnect(trigger: Trigger<WebSocketDisconnectEvent>, mut ws_client: WebSocketClient) {
    let evt = trigger.event();
    warn!("notifs disconnect");

    ws_client.retry(trigger.entity(), evt.request.clone(), RETRY_INTERVAL);
}

#[allow(clippy::too_many_arguments)]
fn on_message(
    trigger: Trigger<WebSocketMessageEvent>,
    mut commands: Commands,
    current_state: Res<State<AppState>>,
    mut app_state: ResMut<NextState<AppState>>,
    mut session_info: Option<ResMut<GameSessionInfo>>,
    mut evw_heartbeat: EventWriter<HeartbeatEvent>,
) {
    let evt = trigger.event();

    match &evt.message {
        Message::Text(value) => {
            info!("received notif from {}: {:?}", evt.uri, value);

            // TODO: error handling
            let notif = serde_json::from_str::<notifs::Notification>(value).unwrap();
            match notif.r#type {
                notifs::NotifType::PlacementRequestV1 => {
                    placement::handle_v1(
                        &mut commands,
                        &current_state,
                        &mut app_state,
                        // TODO: error handling
                        notif.to_message::<notifs::PlacementRequestV1>().unwrap(),
                    );
                }
                notifs::NotifType::ReservationRequestV1 => {
                    reservation::handle_v1(
                        &mut commands,
                        &current_state,
                        session_info.as_mut().unwrap(),
                        // TODO: error handling
                        notif.to_message::<notifs::ReservationRequestV1>().unwrap(),
                        &mut evw_heartbeat,
                    );
                }
            }
        }
        _ => {
            warn!("unexpected notif from {}: {:?}", evt.uri, evt.message);
        }
    }
}

pub fn subscribe<'a>(client: &'a mut WebSocketClient, server_id: Uuid) -> WebSocketBuilder<'a> {
    // TODO: get rid of the need to call into_client_request so we can drop the tungstenite dependency
    let mut notifs_request = format!("{}/gameserver/notifs/v1", HOST)
        .into_client_request()
        .unwrap();
    let headers = notifs_request.headers_mut();
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {}", server_id).parse().unwrap(),
    );

    client
        .connect(notifs_request)
        .on_success(on_success)
        .on_error(on_error)
        .on_message(on_message)
        .on_disconnect(on_disconnect)
}
