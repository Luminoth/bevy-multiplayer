use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{game::OnInGame, GameAssetState, InputState};

#[derive(Debug, Copy, Clone, Component, Reflect, Serialize, Deserialize)]
pub struct Player(ClientId);

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct LocalPlayer;

#[derive(Debug, Component)]
pub struct PlayerCamera;

#[derive(Debug, Default, Copy, Clone, Component, Reflect, Serialize, Deserialize)]
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

pub fn spawn_player(
    commands: &mut Commands,
    client_id: ClientId,
    position: Vec3,
    assets: &GameAssetState,
) {
    info!("spawning player {:?} at {} ...", client_id, position);

    commands.spawn((
        MaterialMeshBundle {
            transform: Transform::from_xyz(position.x, position.y, position.z),
            mesh: assets.player_mesh.clone(),
            material: assets.player_material.clone(),
            ..default()
        },
        RigidBody::KinematicPositionBased,
        GravityScale(2.0),
        Collider::capsule_y(1.0, 1.0),
        ColliderMassProperties::Mass(75.0),
        KinematicCharacterController::default(),
        Name::new(format!("Player {:?}", client_id)),
        Replicated,
        PlayerPhysics::default(),
        Player(client_id),
        OnInGame,
    ));
}

pub fn finish_client_player(
    commands: &mut Commands,
    entity: Entity,
    transform: Transform,
    player: Player,
    assets: &GameAssetState,
    client_id: ClientId,
) {
    info!(
        "finishing player {} ({:?}) at {} ...",
        entity, player.0, transform.translation
    );

    let is_local = player.0 == client_id;

    let mut commands = commands.entity(entity);

    commands.insert((
        MaterialMeshBundle {
            transform,
            mesh: assets.player_mesh.clone(),
            material: assets.player_material.clone(),
            ..default()
        },
        Name::new(format!(
            "Replicated Player ({})",
            if is_local { " Local" } else { "Remote" }
        )),
        OnInGame,
    ));

    if is_local {
        commands.with_children(|parent| {
            parent.spawn((
                Camera3dBundle {
                    transform: Transform::from_xyz(0.0, 1.9, -0.9),
                    projection: PerspectiveProjection {
                        fov: 90.0_f32.to_radians(), // TODO: this should move to settings
                        ..default()
                    }
                    .into(),
                    ..default()
                },
                Name::new("FPS Camera"),
                PlayerCamera,
                OnInGame,
            ));
        });
    }
}

pub fn jump(player_physics: &mut PlayerPhysics) {
    if player_physics.is_grounded() {
        player_physics.velocity.y += JUMP_SPEED;
        player_physics.update_grounded(false);
    }
}

#[derive(Debug)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<JumpEvent>().add_systems(
            FixedUpdate,
            (rotate_player, update_player_physics)
                .chain()
                .run_if(server_or_singleplayer),
        );

        app.register_type::<Player>()
            .register_type::<PlayerPhysics>();

        app.replicate_group::<(Transform, Player, PlayerPhysics)>();
    }
}

fn rotate_player(
    time: Res<Time>,
    mut input_state: ResMut<InputState>,
    mut player_query: Query<&mut Transform, With<LocalPlayer>>,
) {
    // TODO: should the rate of change here be maxed?
    let delta_yaw = -input_state.look.x * time.delta_seconds();

    if let Ok(mut player_transform) = player_query.get_single_mut() {
        player_transform.rotate_y(delta_yaw);
    }

    input_state.look.x = 0.0;
}

#[allow(clippy::type_complexity)]
fn update_player_physics(
    physics_config: Res<RapierConfiguration>,
    time: Res<Time<Fixed>>,
    mut input_state: ResMut<InputState>,
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
        // TODO: we may want to just max() each value instead of normalizing
        let direction = direction.normalize_or_zero();

        if direction.length_squared() > 0.0 {
            player_physics.velocity.x = direction.x * MOVE_SPEED;
            player_physics.velocity.z = direction.z * MOVE_SPEED;
        } else {
            player_physics.velocity.x = 0.0;
            player_physics.velocity.z = 0.0;
        }
    }

    input_state.r#move = Vec2::default();

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
