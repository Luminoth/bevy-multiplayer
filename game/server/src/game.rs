use bevy::prelude::*;

use game_common::{GameState, OnInGame};

use crate::{
    is_not_headless,
    server::{GameServerInfo, GameSessionInfo},
};

#[derive(Debug)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InGame),
            enter_spectate.run_if(is_not_headless),
        );
    }
}

fn enter_spectate(
    mut commands: Commands,
    server_info: Res<GameServerInfo>,
    session_info: Res<GameSessionInfo>,
) {
    info!("entering server spectate game ...");

    commands.insert_resource(ClearColor(Color::BLACK));

    commands.spawn((
        Transform::from_xyz(0.0, 5.0, 0.0),
        Camera3d::default(),
        PerspectiveProjection {
            fov: 90.0_f32.to_radians(),
            ..default()
        },
        Name::new("Server Camera"),
        OnInGame,
    ));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                ..default()
            },
            Name::new("Server UI"),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("Server: {}", server_info.server_id)),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new(format!("Session: {}", session_info.session_id)),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
            ));

            for addr in &server_info.connection_info.v4addrs {
                parent.spawn((
                    Text::new(format!("  {}:{}", addr, server_info.connection_info.port)),
                    TextFont::from_font_size(24.0),
                    TextColor(Color::WHITE),
                ));
            }

            for addr in &server_info.connection_info.v6addrs {
                parent.spawn((
                    Text::new(format!("  {}:{}", addr, server_info.connection_info.port)),
                    TextFont::from_font_size(24.0),
                    TextColor(Color::WHITE),
                ));
            }

            parent.spawn((
                Text::new("Pending Players:"),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
            ));

            // TODO: this list updates over time
            for pending_player in session_info.pending_players.values() {
                parent.spawn((
                    Text::new(format!("  {}", pending_player.user_id)),
                    TextFont::from_font_size(24.0),
                    TextColor(Color::WHITE),
                ));
            }

            parent.spawn((
                Text::new("Active Players:"),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
            ));

            // TODO: this list updates over time
            for active_player in session_info.active_players.values() {
                parent.spawn((
                    Text::new(format!("  {}", active_player.user_id)),
                    TextFont::from_font_size(24.0),
                    TextColor(Color::WHITE),
                ));
            }
        });
}
