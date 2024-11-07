pub mod ball;
mod game;
pub mod network;
pub mod player;
pub mod spawn;
mod world;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use game::{spawn_client_world, GamePlugin, OnInGame, ServerSet};

pub const PROTOCOL_ID: u64 = 0;

// TODO: the issue atm is that there's no way
// to tell the app to go back to its initial state
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, States, Reflect)]
pub enum GameState {
    #[default]
    WaitingForApp,
    LoadAssets,
    SpawnWorld,
    InGame,
}

#[derive(Debug, Default, Copy, Clone, Resource, Reflect, Serialize, Deserialize)]
pub struct InputState {
    pub look: Vec2,
    pub r#move: Vec2,
}

#[derive(Debug, Resource)]
pub struct GameAssetState {
    floor_mesh: Handle<Mesh>,
    floor_material: Handle<StandardMaterial>,

    wall_material: Handle<StandardMaterial>,

    ball_mesh: Handle<Mesh>,
    ball_material: Handle<StandardMaterial>,

    player_mesh: Handle<Mesh>,
    player_material: Handle<StandardMaterial>,
}

pub fn cleanup_state<T>(mut commands: Commands, query: Query<Entity, With<T>>)
where
    T: Component,
{
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}
