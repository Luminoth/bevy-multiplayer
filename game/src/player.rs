use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    game::OnInGame,
    network::{InputUpdateEvent, PlayerJumpEvent},
    GameAssetState, InputState,
};

#[derive(Debug, Copy, Clone, Component, Reflect, Serialize, Deserialize)]
pub struct Player(ClientId);

impl Player {
    #[inline]
    pub fn client_id(&self) -> ClientId {
        self.0
    }
}

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
                Name::new("Player Camera"),
                PlayerCamera,
                OnInGame,
            ));
        });
    }
}

fn rotate_player(time: &Time, input_state: &InputState, transform: &mut Transform) {
    // TODO: should the rate of change here be maxed?
    let delta_yaw = -input_state.look.x * time.delta_seconds();

    transform.rotate_y(delta_yaw);
}

#[allow(clippy::type_complexity)]
fn update_player_physics(
    physics_config: &RapierConfiguration,
    time: &Time<Fixed>,
    input_state: &InputState,
    character_controller: &mut KinematicCharacterController,
    output: Option<&KinematicCharacterControllerOutput>,
    global_transform: &GlobalTransform,
    player_physics: &mut PlayerPhysics,
    gravity_scale: &GravityScale,
) {
    let global_transform = global_transform.compute_transform();

    // update grounded
    let grounded = output.map(|output| output.grounded).unwrap_or_default();
    player_physics.update_grounded(grounded);

    // handle move input
    if player_physics.is_grounded() {
        let direction =
            global_transform.rotation * Vec3::new(input_state.r#move.x, 0.0, -input_state.r#move.y);
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

pub fn handle_input_update(
    time: Res<Time>,
    fixed_time: Res<Time<Fixed>>,
    physics_config: Res<RapierConfiguration>,
    mut evr_input_update: EventReader<FromClient<InputUpdateEvent>>,
    mut player_query: Query<(
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
        &GlobalTransform,
        &mut Transform,
        &mut PlayerPhysics,
        &GravityScale,
        &Player,
    )>,
) {
    for FromClient { client_id, event } in evr_input_update.read() {
        for (
            mut character_controller,
            output,
            global_transform,
            mut transform,
            mut player_physics,
            gravity_scale,
            player,
        ) in &mut player_query
        {
            if player.client_id() == *client_id {
                rotate_player(&time, &event.0, &mut transform);

                update_player_physics(
                    &physics_config,
                    &fixed_time,
                    &event.0,
                    &mut character_controller,
                    output,
                    global_transform,
                    &mut player_physics,
                    gravity_scale,
                );
            }
        }
    }
}

fn jump(player_physics: &mut PlayerPhysics) {
    if player_physics.is_grounded() {
        player_physics.velocity.y += JUMP_SPEED;
        player_physics.update_grounded(false);
    }
}

pub fn handle_jump_event(
    mut evr_jump: EventReader<FromClient<PlayerJumpEvent>>,
    mut player_query: Query<(&mut PlayerPhysics, &Player)>,
) {
    for FromClient {
        client_id,
        event: _,
    } in evr_jump.read()
    {
        for (mut player_physics, player) in &mut player_query {
            if player.client_id() == *client_id {
                jump(&mut player_physics);
            }
        }
    }
}

#[derive(Debug)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<JumpEvent>();

        app.register_type::<Player>()
            .register_type::<PlayerPhysics>();

        app.replicate_group::<(Transform, Player, PlayerPhysics)>();
    }
}
