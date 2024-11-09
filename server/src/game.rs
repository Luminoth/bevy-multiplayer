use bevy::prelude::*;

use game_common::{GameState, OnInGame};

use crate::{
    options::Options,
    server::{GameServerInfo, GameSessionInfo},
};

pub fn is_not_headless(options: Res<Options>) -> bool {
    !options.headless
}

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
    info!("enter server spectate ...");

    commands.insert_resource(ClearColor(Color::BLACK));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, 0.0),
            projection: PerspectiveProjection {
                fov: 90.0_f32.to_radians(),
                ..default()
            }
            .into(),
            ..default()
        },
        Name::new("Server Camera"),
        OnInGame,
    ));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                ..default()
            },
            Name::new("Server UI"),
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                format!("Server: {}", server_info.server_id),
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            parent.spawn(TextBundle::from_section(
                format!("Session: {}", session_info.session_id),
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            // TODO: connection info
        });
}
