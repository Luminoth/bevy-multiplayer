use bevy::prelude::*;

use game_common::{player::PlayerCamera, InputState};

use crate::input;

const PITCH_MAX: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

#[derive(Debug)]
pub struct FpsCameraPlugin;

impl Plugin for FpsCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_fps_camera.after(input::InputSet));
    }
}

fn update_fps_camera(
    time: Res<Time>,
    input_state: Res<InputState>,
    mut camera_query: Query<&mut Transform, With<PlayerCamera>>,
) {
    // TODO: should the rate of change here be maxed?
    let delta_pitch = input_state.look.y * time.delta_seconds();

    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        // can't do this because we need to clamp the pitch
        //camera_transform.rotate_x(delta_pitch);

        let (yaw, pitch, roll) = camera_transform.rotation.to_euler(EulerRot::YXZ);
        let pitch = (pitch + delta_pitch).clamp(-PITCH_MAX, PITCH_MAX);
        camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}
