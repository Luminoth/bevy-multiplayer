use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::input::InputState;

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Player;

#[derive(Debug, Event)]
pub struct JumpEvent;

#[derive(Debug, Default, Component, Reflect)]
pub struct PlayerPhysics {
    pub velocity: Vec3,
    grounded: bool,
}

impl PlayerPhysics {
    #[inline]
    pub fn is_grounded(&self) -> bool {
        self.grounded
    }

    pub fn update_grounded(&mut self, grounded: bool) {
        self.grounded = grounded;
        if self.grounded {
            self.velocity.y = 0.0;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_player_physics(
    physics_config: Res<RapierConfiguration>,
    time: Res<Time<Fixed>>,
    input_state: Res<InputState>,
    mut ev_jump: EventReader<JumpEvent>,
    mut player_query: Query<
        (
            &mut KinematicCharacterController,
            Option<&KinematicCharacterControllerOutput>,
            &GlobalTransform,
            &mut PlayerPhysics,
            &GravityScale,
        ),
        With<Player>,
    >,
) {
    let (mut character_controller, output, player_transform, mut player_physics, gravity_scale) =
        player_query.single_mut();
    let player_transform = player_transform.compute_transform();

    // update grounded
    let grounded = output.map(|output| output.grounded).unwrap_or_default();
    player_physics.update_grounded(grounded);

    // handle move input
    if player_physics.is_grounded() {
        let direction = player_transform.rotation
            * Vec3::new(input_state.r#move().x, 0.0, -input_state.r#move().y);
        if direction.length_squared() > 0.0 {
            player_physics.velocity.x = direction.x * 5.0;
            player_physics.velocity.z = direction.z * 5.0;
        } else {
            player_physics.velocity.x = 0.0;
            player_physics.velocity.z = 0.0;
        }

        if !ev_jump.is_empty() {
            player_physics.velocity.y += 10.0;
            ev_jump.clear();
        }
    }

    // apply gravity
    player_physics.velocity.y += physics_config.gravity.y * gravity_scale.0 * time.delta_seconds();
    player_physics.velocity.y = player_physics.velocity.y.max(-50.0);

    // move
    let translation = character_controller
        .translation
        .get_or_insert(Vec3::default());
    *translation += player_physics.velocity * time.delta_seconds();
}
