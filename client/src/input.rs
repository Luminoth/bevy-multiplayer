use bevy::{input::gamepad::GamepadEvent, prelude::*};

use game::{player::JumpEvent, AppState, InputState};

// TODO: move to a settings resource
const INVERT_LOOK: bool = true;

#[derive(Debug)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((handle_gamepad_events, update_gamepad).chain()).run_if(in_state(AppState::InGame)),
        );
    }
}

fn handle_gamepad_events(mut evr_gamepad: EventReader<GamepadEvent>) {
    for _ev in evr_gamepad.read() {
        // TODO: handle connection events
    }
}

fn update_gamepad(
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<ButtonInput<GamepadButton>>,
    mut input_state: ResMut<InputState>,
    mut ev_jump: EventWriter<JumpEvent>,
) {
    let gamepad = Gamepad { id: 0 };

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
        input_state.look = Vec2::new(x, if INVERT_LOOK { -1.0 } else { 1.0 } * y);
    } else {
        input_state.look = Vec2::default();
    }

    let south_button = GamepadButton {
        gamepad,
        button_type: GamepadButtonType::South,
    };

    if buttons.just_pressed(south_button) {
        ev_jump.send(JumpEvent);
    }
}
