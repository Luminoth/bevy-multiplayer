mod ball;
mod game;
pub mod player;

use bevy::prelude::*;

pub use game::GamePlugin;

pub const PROTOCOL_ID: u64 = 0;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, States, Reflect)]
pub enum AppState {
    #[cfg(feature = "client")]
    #[cfg(not(feature = "server"))]
    #[default]
    MainMenu,

    #[cfg(feature = "client")]
    #[cfg(not(feature = "server"))]
    ConnectToServer,

    #[cfg(feature = "client")]
    #[cfg(not(feature = "server"))]
    WaitForConnect,

    #[cfg(feature = "server")]
    #[cfg(not(feature = "client"))]
    #[default]
    WaitForPlacement,

    #[cfg(feature = "server")]
    #[cfg(not(feature = "client"))]
    InitServer,

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
