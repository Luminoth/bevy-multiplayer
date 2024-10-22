#![cfg(not(feature = "server"))]

use bevy::{input::gamepad::GamepadEvent, prelude::*};
use bevy_rapier3d::prelude::*;

pub fn poll_gamepad(
    axes: Res<Axis<GamepadAxis>>,
    time: Res<Time>,
    mut player_query: Query<
        (&mut KinematicCharacterController, &mut Velocity),
        With<crate::game::Player>,
    >,
) {
    let gamepad = Gamepad { id: 0 };
    let (mut character_controller, _) = player_query.single_mut();

    // left stick (move)
    let axis_lx = GamepadAxis {
        gamepad,
        axis_type: GamepadAxisType::LeftStickX,
    };
    let axis_ly = GamepadAxis {
        gamepad,
        axis_type: GamepadAxisType::LeftStickY,
    };

    // TODO: don't update the controller, update the velocity
    // then we want a step that adds in gravity and calculates the translation from the velocity
    {
        let translation = character_controller
            .translation
            .get_or_insert(Vec3::default());
        if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
            translation.x = x * time.delta_seconds() * 5.0;
            translation.z = -y * time.delta_seconds() * 5.0;
        } else {
            translation.x = 0.0;
            translation.z = 0.0;
        }
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

    if let (Some(_x), Some(_y)) = (axes.get(axis_rx), axes.get(axis_ry)) {
        // TODO: move look around
    }
}

pub fn handle_gamepad_events(mut evr_gamepad: EventReader<GamepadEvent>) {
    for _ev in evr_gamepad.read() {
        // TODO: handle connection events
    }
}
