use bevy::prelude::*;

use game_common::{player, spawn::SpawnPoint, GameAssetState, GameState};

#[derive(Debug)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), enter);
    }
}

fn enter(
    mut commands: Commands,
    assets: Res<GameAssetState>,
    _spawnpoints: Query<&GlobalTransform, With<SpawnPoint>>,
) {
    info!("enter game (client) ...");

    // spawn player
    // TODO: the world has to spawn before we can spawn the player using a spawnpoint
    //let spawnpoint = spawnpoints.iter().next().unwrap();
    player::spawn_local_player(
        &mut commands,
        Vec3::new(-5.0, 2.1, 5.0), /*spawnpoint.translation()*/
        &assets,
    );
}
