use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{game::OnInGame, InputState};

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct LocalPlayer;

#[derive(Debug, Component)]
pub struct PlayerCamera;

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

#[derive(Debug, Event)]
pub struct JumpEvent;

const MOVE_SPEED: f32 = 5.0;
const JUMP_SPEED: f32 = 10.0;
const TERMINAL_VELOCITY: f32 = 50.0;

pub fn spawn_camera(commands: &mut Commands, position: Vec3) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(position.x, position.y, position.z),
            projection: PerspectiveProjection {
                fov: 90.0_f32.to_radians(), // TODO: this should move to settings
                ..default()
            }
            .into(),
            ..default()
        },
        Name::new("Player Camera"),
        PlayerCamera,
        OnInGame,
    ));
}

pub fn spawn_player(
    commands: &mut Commands,
    position: Vec3,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
) {
    commands.spawn((
        MaterialMeshBundle {
            transform: Transform::from_xyz(position.x, position.y, position.z),
            mesh,
            material,
            ..default()
        },
        RigidBody::KinematicPositionBased,
        PlayerPhysics::default(),
        GravityScale(2.0),
        Collider::capsule_y(1.0, 1.0),
        ColliderMassProperties::Mass(75.0),
        KinematicCharacterController::default(),
        Name::new("Player"),
        LocalPlayer,
        OnInGame,
    ));
}

#[derive(Debug)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<JumpEvent>()
            .add_systems(FixedUpdate, update_player_physics);

        app.register_type::<PlayerPhysics>();

        app.replicate_group::<(Transform, LocalPlayer)>();
    }
}

#[allow(clippy::type_complexity)]
fn update_player_physics(
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
        With<LocalPlayer>,
    >,
) {
    let Ok((mut character_controller, output, player_transform, mut player_physics, gravity_scale)) =
        player_query.get_single_mut()
    else {
        return;
    };

    let player_transform = player_transform.compute_transform();

    // update grounded
    let grounded = output.map(|output| output.grounded).unwrap_or_default();
    player_physics.update_grounded(grounded);

    // handle move input
    if player_physics.is_grounded() {
        let direction =
            player_transform.rotation * Vec3::new(input_state.r#move.x, 0.0, -input_state.r#move.y);
        if direction.length_squared() > 0.0 {
            player_physics.velocity.x = direction.x * MOVE_SPEED;
            player_physics.velocity.z = direction.z * MOVE_SPEED;
        } else {
            player_physics.velocity.x = 0.0;
            player_physics.velocity.z = 0.0;
        }

        if !ev_jump.is_empty() {
            player_physics.velocity.y += JUMP_SPEED;
            ev_jump.clear();
        }
    }

    // apply gravity
    player_physics.velocity.y += physics_config.gravity.y * gravity_scale.0 * time.delta_seconds();
    player_physics.velocity.y = player_physics
        .velocity
        .y
        .clamp(-TERMINAL_VELOCITY, TERMINAL_VELOCITY);

    // move
    let translation = character_controller
        .translation
        .get_or_insert(Vec3::default());
    *translation += player_physics.velocity * time.delta_seconds();
}
