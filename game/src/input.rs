#![cfg(not(feature = "server"))]

use bevy::{input::gamepad::GamepadEvent, prelude::*};

use crate::game::{MainCamera, Player, PlayerPhysics};

const INVERT_LOOK: bool = true;

pub fn poll_gamepad(
    axes: Res<Axis<GamepadAxis>>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    mut player_query: Query<(&GlobalTransform, &mut PlayerPhysics), With<Player>>,
) {
    let mut camera_transform = camera_query.single_mut();

    let gamepad = Gamepad { id: 0 };
    let (player_transform, mut player_physics) = player_query.single_mut();
    let player_transform = player_transform.compute_transform();

    // left stick (move)
    let axis_lx = GamepadAxis {
        gamepad,
        axis_type: GamepadAxisType::LeftStickX,
    };
    let axis_ly = GamepadAxis {
        gamepad,
        axis_type: GamepadAxisType::LeftStickY,
    };

    player_physics.velocity.x = 0.0;
    player_physics.velocity.z = 0.0;

    if player_physics.is_grounded() {
        if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
            let direction = player_transform.rotation * Vec3::new(x, 0.0, -y);
            if direction.length_squared() > 0.0 {
                player_physics.velocity.x = direction.x * 5.0;
                player_physics.velocity.z = direction.z * 5.0;
            }
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

        let (yaw, pitch, roll) = camera_transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;
        let pitch = (pitch + delta_pitch).clamp(
            -(std::f32::consts::FRAC_PI_2 - 0.01),
            std::f32::consts::FRAC_PI_2 - 0.01,
        );

        camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}

pub fn handle_gamepad_events(mut evr_gamepad: EventReader<GamepadEvent>) {
    for _ev in evr_gamepad.read() {
        // TODO: handle connection events
    }
}
