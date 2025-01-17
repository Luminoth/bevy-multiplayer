use bevy::prelude::*;

use crate::server::{ActivePlayer, PendingPlayer};

use game_common::{GameState, OnInGame};

use crate::{
    is_not_headless,
    server::{GameServerInfo, GameSessionInfo},
};

// TODO: spectator UI would be much less expensive if it were EGUI

#[derive(Debug, Component)]
struct PendingPlayerList;

#[derive(Debug, Component)]
struct ActivePlayerList;

#[derive(Debug)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::InGame),
            enter_spectate.run_if(is_not_headless),
        )
        .add_systems(
            Update,
            update_spectate_ui
                .run_if(in_state(GameState::InGame))
                .run_if(is_not_headless),
        );
    }
}

fn enter_spectate(
    mut commands: Commands,
    server_info: Res<GameServerInfo>,
    session_info: Res<GameSessionInfo>,
    pending_players: Query<&PendingPlayer>,
    active_players: Query<&ActivePlayer>,
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

            let mut pending_players_str = String::new();
            for pending_player in &pending_players {
                pending_players_str.push_str(&format!("  {}", pending_player.user_id));
            }

            parent.spawn((
                Text::new(pending_players_str),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
                PendingPlayerList,
            ));

            parent.spawn((
                Text::new("Active Players:"),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
            ));

            let mut active_players_str = String::new();
            for active_player in &active_players {
                active_players_str.push_str(&format!("  {}", active_player.user_id));
            }

            parent.spawn((
                Text::new(active_players_str),
                TextFont::from_font_size(24.0),
                TextColor(Color::WHITE),
                ActivePlayerList,
            ));
        });
}

// TODO: this is terrible, we should only update these values if they've changed
fn update_spectate_ui(
    pending_players: Query<&PendingPlayer>,
    mut pending_player_list: Query<&mut Text, (With<PendingPlayerList>, Without<ActivePlayerList>)>,
    active_players: Query<&ActivePlayer>,
    mut active_player_list: Query<&mut Text, (With<ActivePlayerList>, Without<PendingPlayerList>)>,
) {
    let mut pending_players_str = String::new();
    for pending_player in &pending_players {
        pending_players_str.push_str(&format!("  {}", pending_player.user_id));
    }

    pending_player_list.single_mut().0 = pending_players_str;

    let mut active_players_str = String::new();
    for active_player in &active_players {
        active_players_str.push_str(&format!("  {}", active_player.user_id));
    }

    active_player_list.single_mut().0 = active_players_str;
}
