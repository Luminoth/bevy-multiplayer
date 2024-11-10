use bevy::{input::common_conditions::*, prelude::*};

use game_common::{
    dynamic, network::PlayerClientId, player, spawn::SpawnPoint, GameAssetState, GameState,
    ServerSet,
};

use crate::game_menu;

pub fn is_local_game(client_id: Res<PlayerClientId>) -> bool {
    client_id.is_local()
}

#[derive(Debug)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InGame),
            enter_local.after(ServerSet).run_if(is_local_game),
        )
        .add_systems(
            Update,
            toggle_game_menu
                .run_if(in_state(GameState::InGame))
                .run_if(input_just_released(KeyCode::Escape)),
        );
    }
}

#[allow(clippy::type_complexity)]
fn enter_local(
    mut commands: Commands,
    client_id: Res<PlayerClientId>,
    assets: Res<GameAssetState>,
    spawnpoints: Query<&GlobalTransform, With<SpawnPoint>>,
    dynamics: Query<(Entity, &Transform, &dynamic::Dynamic), Without<GlobalTransform>>,
    players: Query<(Entity, &Transform, &player::Player), Without<GlobalTransform>>,
) {
    info!("finishing local game ...");

    let spawnpoint = spawnpoints.iter().next().unwrap();
    player::spawn_player(
        &mut commands,
        client_id.get_client_id(),
        spawnpoint.translation(),
        &assets,
    );

    game_common::spawn_client_world(
        &mut commands,
        client_id.get_client_id(),
        &assets,
        &dynamics,
        &players,
    );
}

fn toggle_game_menu(mut visibility: Query<&mut Visibility, With<game_menu::GameMenu>>) {
    let mut current = visibility.single_mut();
    if *current == Visibility::Visible {
        *current = Visibility::Hidden;
    } else {
        *current = Visibility::Visible;
    }
}
