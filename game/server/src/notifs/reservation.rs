use bevy::prelude::*;
use bevy_mod_reqwest::*;

use internal::notifs;

use crate::{
    api,
    orchestration::Orchestration,
    server::{GameServerInfo, GameSessionInfo, PendingPlayer},
    AppState,
};

pub fn handle_v1(
    client: &mut BevyReqwest,
    current_state: &AppState,
    orchestration: &Orchestration,
    server_info: &GameServerInfo,
    session_info: &mut GameSessionInfo,
    request: notifs::ReservationRequestV1,
) {
    if *current_state != AppState::InGame {
        warn!("ignoring unexpected reservation request!");
        return;
    }

    if session_info.player_count() + request.player_ids.len() > session_info.max_players as usize {
        warn!(
            "ignoring reservation request with too many players: {}",
            request.player_ids.len()
        );
        return;
    }

    info!("reserving player slots: {:?}", request.player_ids);

    for player_id in request.player_ids {
        session_info
            .pending_players
            .insert(player_id, PendingPlayer::new(player_id));
    }

    api::heartbeat(
        client,
        server_info.server_id,
        server_info.connection_info.clone(),
        (*current_state).into(),
        orchestration.as_api_type(),
        Some(&session_info),
    )
    .unwrap();
}
