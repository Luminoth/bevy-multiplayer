use bevy::prelude::*;

use game::{
    player::{LocalPlayer, PlayerCamera},
    InputState,
};

const PITCH_MAX: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

#[derive(Debug)]
pub struct FpsCameraPlugin;

impl Plugin for FpsCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_fps_camera);
    }
}

#[allow(clippy::type_complexity)]
fn update_fps_camera(
    time: Res<Time>,
    input_state: Res<InputState>,
    mut transforms_query: ParamSet<(
        Query<&mut Transform, With<PlayerCamera>>,
        Query<&mut Transform, With<LocalPlayer>>,
    )>,
) {
    let delta_yaw = -input_state.look.x * time.delta_seconds();
    let delta_pitch = input_state.look.y * time.delta_seconds();

    // TODO: does this belong in a player method?
    let mut player_query = transforms_query.p1();
    if let Ok(mut player_transform) = player_query.get_single_mut() {
        // TODO: should this be done on the physics step?
        player_transform.rotate_y(delta_yaw);
    }

    let mut camera_query = transforms_query.p0();
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        // can't do this because we need to clamp the pitch
        //camera_transform.rotate_x(delta_pitch);

        let (yaw, pitch, roll) = camera_transform.rotation.to_euler(EulerRot::YXZ);
        let pitch = (pitch + delta_pitch).clamp(-PITCH_MAX, PITCH_MAX);
        camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}
