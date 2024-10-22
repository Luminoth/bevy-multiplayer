#![cfg(not(feature = "server"))]

use bevy::{input::gamepad::GamepadEvent, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::game::MainCamera;

const INVERT_LOOK: bool = true;

pub fn poll_gamepad(
    axes: Res<Axis<GamepadAxis>>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut player_query: Query<
        (&mut KinematicCharacterController, &mut Velocity),
        With<crate::game::Player>,
    >,
) {
    let mut camera = camera_query.single_mut();

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

    if let (Some(x), Some(y)) = (axes.get(axis_rx), axes.get(axis_ry)) {
        let delta_yaw = -x * 4.0 * time.delta_seconds();
        let delta_pitch = if INVERT_LOOK { -1.0 } else { 1.0 } * y * 4.0 * time.delta_seconds();

        let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;
        let pitch = (pitch + delta_pitch).clamp(
            -(std::f32::consts::FRAC_PI_2 - 0.01),
            std::f32::consts::FRAC_PI_2 - 0.01,
        );

        camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}

pub fn handle_gamepad_events(mut evr_gamepad: EventReader<GamepadEvent>) {
    for _ev in evr_gamepad.read() {
        // TODO: handle connection events
    }
}
