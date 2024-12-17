use bevy::{
    input::{
        gamepad::{GamepadConnection, GamepadEvent},
        mouse::MouseMotion,
    },
    prelude::*,
    window::PrimaryWindow,
};

use game_common::{GameState, InputState};

use crate::{game_menu, Settings};

#[derive(Debug, Resource)]
struct ConnectedGamepad(Gamepad);

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct InputSet;

#[derive(Debug, Event)]
pub struct JumpPressedEvent;

#[derive(Debug)]
pub struct InputPlugin;

fn should_update_input(
    window_query: Query<&Window, With<PrimaryWindow>>,
    game_menu_query: Query<&Visibility, With<game_menu::GameMenu>>,
) -> bool {
    if let Ok(window) = window_query.get_single() {
        if !window.focused {
            return false;
        }
    } else {
        return false;
    }

    if let Ok(visibility) = game_menu_query.get_single() {
        if *visibility == Visibility::Visible {
            return false;
        }
    }

    true
}

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<JumpPressedEvent>()
            // TODO: this is running before bevy picks up the set of gamepads
            .add_systems(Startup, list_gamepads)
            .add_systems(
                Update,
                (
                    handle_gamepad_events,
                    (update_mnk, (update_gamepad.after(handle_gamepad_events)))
                        .run_if(should_update_input)
                        .run_if(in_state(GameState::InGame)),
                )
                    .in_set(InputSet),
            );
    }
}

fn list_gamepads(gamepads: Res<Gamepads>) {
    info!("{} connected gamepads:", gamepads.iter().count());
    for gamepad in gamepads.iter() {
        info!(
            "{:?}: {}",
            gamepad,
            gamepads.name(gamepad).unwrap_or("unknown")
        );
    }
}

fn handle_gamepad_events(
    mut commands: Commands,
    gamepad: Option<Res<ConnectedGamepad>>,
    mut evr_gamepad: EventReader<GamepadEvent>,
) {
    for evt in evr_gamepad.read() {
        let GamepadEvent::Connection(evt_conn) = evt else {
            continue;
        };

        match &evt_conn.connection {
            GamepadConnection::Connected(info) => {
                info!(
                    "gamepad connected: {:?}, name: {}",
                    evt_conn.gamepad, info.name,
                );

                if gamepad.is_none() {
                    info!("using gamepad {:?}", evt_conn.gamepad);
                    commands.insert_resource(ConnectedGamepad(evt_conn.gamepad));
                }
            }
            GamepadConnection::Disconnected => {
                warn!("gamepad disconnected: {:?}", evt_conn.gamepad);

                if let Some(ConnectedGamepad(gamepad)) = gamepad.as_deref() {
                    if *gamepad == evt_conn.gamepad {
                        commands.remove_resource::<ConnectedGamepad>();
                    }
                }
            }
        }
    }
}

fn update_mnk(
    keys: Res<ButtonInput<KeyCode>>,
    mut evr_motion: EventReader<MouseMotion>,
    mut input_state: ResMut<InputState>,
    settings: Res<Settings>,
    mut evw_jump: EventWriter<JumpPressedEvent>,
) {
    if !settings.mnk.enabled {
        return;
    }

    let mut r#move = Vec2::default();
    if keys.pressed(KeyCode::KeyW) {
        r#move.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        r#move.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        r#move.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        r#move.x += 1.0;
    }

    input_state.r#move += r#move;

    if keys.just_pressed(KeyCode::Space) {
        evw_jump.send(JumpPressedEvent);
    }

    let mut look = Vec2::default();
    for evt in evr_motion.read() {
        look += Vec2::new(
            evt.delta.x,
            if settings.mnk.invert_look { -1.0 } else { 1.0 } * -evt.delta.y,
        ) * settings.mnk.mouse_sensitivity;
    }

    input_state.look += look;
}

fn update_gamepad(
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<ButtonInput<GamepadButton>>,
    settings: Res<Settings>,
    gamepad: Option<Res<ConnectedGamepad>>,
    mut input_state: ResMut<InputState>,
    mut evw_jump: EventWriter<JumpPressedEvent>,
) {
    if !settings.gamepad.enabled {
        return;
    }

    let Some(&ConnectedGamepad(gamepad)) = gamepad.as_deref() else {
        return;
    };

    // left stick (move)
    let axis_lx = GamepadAxis {
        gamepad,
        axis_type: GamepadAxisType::LeftStickX,
    };
    let axis_ly = GamepadAxis {
        gamepad,
        axis_type: GamepadAxisType::LeftStickY,
    };

    if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
        input_state.r#move += Vec2::new(x, y);
    }

    // right stick (look)
    let axis_rx = GamepadAxis {
        gamepad,
        axis_type: GamepadAxisType::RightStickX,
    };
    let axis_ry = GamepadAxis {
        gamepad,
        axis_type: GamepadAxisType::RightStickY,
    };

    if let (Some(x), Some(y)) = (axes.get(axis_rx), axes.get(axis_ry)) {
        input_state.look += Vec2::new(
            x,
            if settings.gamepad.invert_look {
                -1.0
            } else {
                1.0
            } * y,
        ) * settings.gamepad.look_sensitivity;
    }

    let south_button = GamepadButton {
        gamepad,
        button_type: GamepadButtonType::South,
    };

    if buttons.just_pressed(south_button) {
        evw_jump.send(JumpPressedEvent);
    }
}
