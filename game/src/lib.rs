mod ball;
mod game;
pub mod player;
mod spawn;
mod world;

use bevy::prelude::*;

pub use game::GamePlugin;

pub const PROTOCOL_ID: u64 = 0;

// TODO: the issue atm is that there's no way
// to tell the app to go back to its initial state
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, States, Reflect)]
pub enum GameState {
    #[default]
    WaitingForApp,
    LoadAssets,
    InGame,
}

// TODO: should this be split into separate resources?
#[derive(Debug, Default, Resource, Reflect)]
pub struct InputState {
    pub look: Vec2,
    pub r#move: Vec2,
}

pub fn cleanup_state<T>(mut commands: Commands, query: Query<Entity, With<T>>)
where
    T: Component,
{
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}
