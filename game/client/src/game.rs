use bevy::{input::common_conditions::*, prelude::*, window::PrimaryWindow};

use game_common::{
    network::PlayerClientId, player, spawn::SpawnPoint, GameAssetState, GameState, ServerSet,
};

use crate::{game_menu, recenter_cursor, show_cursor};

pub fn is_local_game(client_id: Res<PlayerClientId>) -> bool {
    client_id.is_local()
}

#[derive(Debug)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InGame),
            (
                enter,
                (spawn_local_player, finish_local)
                    .chain()
                    .after(ServerSet)
                    .run_if(is_local_game),
            ),
        )
        .add_systems(
            Update,
            toggle_game_menu
                .run_if(in_state(GameState::InGame))
                .run_if(input_just_released(KeyCode::Escape)),
        );
    }
}

fn enter(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    info!("entering client game ...");

    show_cursor(window_query.get_single_mut().as_mut().ok(), false);
}

#[allow(clippy::type_complexity)]
fn spawn_local_player(
    mut commands: Commands,
    client_id: Res<PlayerClientId>,
    assets: Res<GameAssetState>,
    spawnpoints: Query<&GlobalTransform, With<SpawnPoint>>,
) {
    info!("spawning local player ...");

    let spawnpoint = spawnpoints.iter().next().unwrap();
    let entity = player::spawn_player(
        &mut commands,
        client_id.get_client_id(),
        spawnpoint.translation(),
        &assets,
    );

    game_common::player::finish_local_player(&mut commands, entity);
}

#[allow(clippy::type_complexity)]
fn finish_local(mut commands: Commands) {
    info!("finishing local game ...");

    game_common::spawn_client_world(&mut commands);
}

fn toggle_game_menu(
    mut visibility: Query<&mut Visibility, With<game_menu::GameMenu>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut current = visibility.single_mut();
    if *current == Visibility::Visible {
        *current = Visibility::Hidden;

        show_cursor(window_query.get_single_mut().as_mut().ok(), false);
    } else {
        *current = Visibility::Visible;

        show_cursor(window_query.get_single_mut().as_mut().ok(), true);
        recenter_cursor(window_query.get_single_mut().as_mut().ok());
    }
}
