#![cfg(not(feature = "server"))]

use bevy::prelude::*;

use crate::{input::InputState, player::Player};

#[derive(Debug, Component)]
pub struct MainCamera;

#[allow(clippy::type_complexity)]
pub fn update_camera(
    time: Res<Time>,
    input_state: Res<InputState>,
    mut transforms_query: ParamSet<(
        Query<&mut Transform, With<MainCamera>>,
        Query<&mut Transform, With<Player>>,
    )>,
) {
    let delta_yaw = -input_state.look().x * 4.0 * time.delta_seconds();
    let delta_pitch = input_state.look().y * 4.0 * time.delta_seconds();

    let mut player_query = transforms_query.p1();
    let mut player_transform = player_query.single_mut();

    // TODO: should this be done on the physics step?
    player_transform.rotate_y(delta_yaw);

    let mut camera_query = transforms_query.p0();
    let mut camera_transform = camera_query.single_mut();

    let (yaw, pitch, roll) = camera_transform.rotation.to_euler(EulerRot::YXZ);
    let yaw = yaw + delta_yaw; // TODO: don't do this once the camera is under the player
    let pitch = (pitch + delta_pitch).clamp(
        -(std::f32::consts::FRAC_PI_2 - 0.01),
        std::f32::consts::FRAC_PI_2 - 0.01,
    );

    camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
}
