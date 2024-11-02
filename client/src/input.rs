use bevy::{
    input::{
        gamepad::{GamepadConnection, GamepadEvent},
        mouse::MouseMotion,
    },
    prelude::*,
};

use game_common::{player::JumpEvent, GameState, InputState};

use crate::Settings;

#[derive(Debug, Resource)]
struct ConnectedGamepad(Gamepad);

#[derive(Debug)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, list_gamepads).add_systems(
            Update,
            (update_mnk, (handle_gamepad_events, update_gamepad).chain())
                .run_if(in_state(GameState::InGame)),
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
    mut evw_jump: EventWriter<JumpEvent>,
) {
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

    input_state.r#move = r#move;

    if keys.just_pressed(KeyCode::Space) {
        evw_jump.send(JumpEvent);
    }

    let mut look = Vec2::default();
    for evt in evr_motion.read() {
        look += Vec2::new(
            evt.delta.x,
            if settings.mnk.invert_look { -1.0 } else { 1.0 } * -evt.delta.y,
        ) * settings.mnk.mouse_sensitivity;
    }
    input_state.look = look;
}

fn update_gamepad(
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<ButtonInput<GamepadButton>>,
    settings: Res<Settings>,
    gamepad: Option<Res<ConnectedGamepad>>,
    mut input_state: ResMut<InputState>,
    mut evw_jump: EventWriter<JumpEvent>,
) {
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
        input_state.r#move = Vec2::new(x, y);
    } else {
        input_state.r#move = Vec2::default();
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
        input_state.look = Vec2::new(
            x,
            if settings.gamepad.invert_look {
                -1.0
            } else {
                1.0
            } * y,
        ) * settings.gamepad.look_sensitivity;
    } else {
        input_state.look = Vec2::default();
    }

    let south_button = GamepadButton {
        gamepad,
        button_type: GamepadButtonType::South,
    };

    if buttons.just_pressed(south_button) {
        evw_jump.send(JumpEvent);
    }
}
