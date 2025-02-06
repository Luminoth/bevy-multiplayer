use bevy::prelude::*;

use common::GameSettings;
use game_common::server::GameSessionInfo;
use internal::notifs;

use crate::AppState;

pub fn handle_v1(
    commands: &mut Commands,
    current_state: &AppState,
    app_state: &mut NextState<AppState>,
    request: notifs::PlacementRequestV1,
) {
    if *current_state != AppState::WaitForPlacement {
        warn!("ignoring unexpected placement request!");
        return;
    }

    // TODO: should come from the placement request
    // (as matchtype or something we can look up settings for)
    let game_settings = GameSettings::default();

    if request.player_ids.len() > game_settings.max_players as usize {
        warn!(
            "ignoring placement request with too many players: {}",
            request.player_ids.len()
        );
        return;
    }

    info!(
        "starting session {}: {:?}",
        request.game_session_id, request.player_ids
    );

    let session_info = GameSessionInfo::new(
        commands,
        request.game_session_id,
        &game_settings,
        request.player_ids,
    );

    commands.insert_resource(session_info);

    app_state.set(AppState::InitServer);
}
