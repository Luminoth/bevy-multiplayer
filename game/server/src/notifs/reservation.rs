use bevy::prelude::*;

use game_common::server::GameSessionInfo;
use internal::notifs;

use crate::{server::HeartbeatEvent, AppState};

pub fn handle_v1(
    commands: &mut Commands,
    current_state: &AppState,
    session_info: &mut GameSessionInfo,
    request: notifs::ReservationRequestV1,
    evw_heartbeat: &mut EventWriter<HeartbeatEvent>,
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
        session_info.reserve_player(commands, player_id);
    }

    evw_heartbeat.send_default();
}
