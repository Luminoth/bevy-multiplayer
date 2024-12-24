use bevy::prelude::*;
use bevy_mod_reqwest::*;
use bevy_mod_websocket::*;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use uuid::Uuid;

use internal::notifs;

use crate::{
    api,
    orchestration::Orchestration,
    server::{GameServerInfo, GameSessionInfo},
    AppState,
};

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

#[allow(clippy::too_many_arguments)]
fn on_message(
    trigger: Trigger<WebSocketMessageEvent>,
    mut commands: Commands,
    mut client: BevyReqwest,
    current_state: Res<State<AppState>>,
    mut app_state: ResMut<NextState<AppState>>,
    orchestration: Res<Orchestration>,
    server_info: Res<GameServerInfo>,
    session_info: Option<ResMut<GameSessionInfo>>,
) {
    let evt = trigger.event();

    match &evt.message {
        Message::Text(value) => {
            info!("received notif from {}: {:?}", evt.uri, value);

            // TODO: error handling
            let notif = serde_json::from_str::<notifs::Notification>(value).unwrap();
            match notif.r#type {
                notifs::NotifType::PlacementRequestV1 => {
                    if *current_state != AppState::WaitForPlacement {
                        warn!("ignoring unexpected placement request!");
                        return;
                    }

                    // TODO: error handling
                    let message = notif.to_message::<notifs::PlacementRequestV1>().unwrap();

                    // TODO: should come from the placement request
                    // (as matchtype or something we can look up settings for)
                    let game_settings = internal::GameSettings::default();

                    if message.player_ids.len() > game_settings.max_players as usize {
                        warn!(
                            "ignoring placement request with too many players: {}",
                            message.player_ids.len()
                        );
                        return;
                    }

                    info!(
                        "starting session {}: {:?}",
                        message.game_session_id, message.player_ids
                    );

                    let session_info = GameSessionInfo::new(
                        message.game_session_id,
                        &game_settings,
                        message.player_ids,
                    );

                    commands.insert_resource(session_info);

                    app_state.set(AppState::InitServer);
                }
                notifs::NotifType::ReservationRequestV1 => {
                    if *current_state != AppState::InGame {
                        warn!("ignoring unexpected reservation request!");
                        return;
                    }

                    let mut session_info = session_info.unwrap();

                    // TODO: error handling
                    let message = notif.to_message::<notifs::ReservationRequestV1>().unwrap();

                    if session_info.player_count() + message.player_ids.len()
                        > session_info.max_players as usize
                    {
                        warn!(
                            "ignoring reservation request with too many players: {}",
                            message.player_ids.len()
                        );
                        return;
                    }

                    info!("reserving player slots: {:?}", message.player_ids);

                    for player_id in message.player_ids {
                        session_info.pending_player_ids.insert(player_id);
                    }

                    api::heartbeat(
                        &mut client,
                        server_info.server_id,
                        server_info.connection_info.clone(),
                        (**current_state).into(),
                        orchestration.as_api_type(),
                        Some(&session_info),
                    )
                    .unwrap();
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
