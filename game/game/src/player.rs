use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_tnua::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{game::OnInGame, network, GameAssetState, InputState};

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
}

#[derive(Debug, Default, Component)]
pub struct LastInput {
    pub input_state: InputState,
    pub jump: bool,
}

// TODO: if these moved to a resource
// they'd be easier to fudge for testing
const MOVE_SPEED: f32 = 10.0;
//const JUMP_SPEED: f32 = 15.0;
//const TERMINAL_VELOCITY: f32 = 50.0;
const GRAVITY_SCALE: f32 = 4.0;
const HEIGHT: f32 = 2.0;
const MASS: f32 = 75.0;

pub fn spawn_player(
    commands: &mut Commands,
    client_id: ClientId,
    position: Vec3,
    assets: &GameAssetState,
) -> Entity {
    info!("spawning player {:?} at {} ...", client_id, position);

    commands
        .spawn((
            Mesh3d(assets.player_mesh.clone()),
            MeshMaterial3d(assets.player_material.clone()),
            Transform::from_xyz(position.x, position.y, position.z),
            //RigidBody::Kinematic,
            RigidBody::Dynamic,
            Collider::capsule(HEIGHT * 0.5, HEIGHT * 0.5),
            Mass(MASS),
            GravityScale(GRAVITY_SCALE),
            TnuaController::default(),
            // TODO:
            //TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
            //LockedAxes::ROTATION_LOCKED,
            Name::new(format!("Player {:?}", client_id)),
            Replicated,
            PlayerPhysics::default(),
            LastInput::default(),
            Player(client_id),
            OnInGame,
        ))
        .id()
}

pub fn despawn_player(commands: &mut Commands, entity: Entity) {
    info!("despawning player {} ...", entity);

    commands.entity(entity).despawn_recursive();
}

pub fn finish_client_player(
    commands: &mut Commands,
    client_id: ClientId,
    assets: &GameAssetState,
    entity: Entity,
    transform: &Transform,
    player: &Player,
) {
    info!(
        "finishing player {} ({:?}:{:?}) at {} ...",
        entity, player.0, client_id, transform.translation
    );

    let is_local = player.0 == client_id;

    let mut ec = commands.entity(entity);
    ec.insert((
        Mesh3d(assets.player_mesh.clone()),
        MeshMaterial3d(assets.player_material.clone()),
        Name::new(format!(
            "Replicated Player ({})",
            if is_local { " Local" } else { "Remote" }
        )),
        OnInGame,
    ));

    if is_local {
        finish_local_player(commands, entity);
    }
}

pub fn finish_local_player(commands: &mut Commands, entity: Entity) {
    info!("finishing local player {} ...", entity);

    let mut ec = commands.entity(entity);
    ec.insert(LocalPlayer);

    ec.with_children(|parent| {
        parent.spawn((
            Transform::from_xyz(0.0, 1.9, -0.9),
            Camera3d::default(),
            PerspectiveProjection {
                fov: 90.0_f32.to_radians(), // TODO: this should move to settings
                ..default()
            },
            Name::new("Player Camera"),
            PlayerCamera,
            OnInGame,
        ));
    });
}

#[derive(Debug)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            finish_client_players
                .after(ClientSet::Receive)
                .run_if(client_connected),
        )
        .add_systems(Update, rotate_player)
        .add_systems(FixedUpdate, update_player_physics);

        app.register_type::<Player>()
            .register_type::<PlayerPhysics>();

        app.replicate_group::<(Transform, Player, PlayerPhysics)>();
    }
}

fn finish_client_players(
    mut commands: Commands,
    client_id: Res<network::PlayerClientId>,
    assets: Option<Res<GameAssetState>>,
    players: Query<(Entity, &Transform, &Player), Without<Mesh3d>>,
) {
    let Some(assets) = assets else {
        return;
    };

    for (entity, transform, player) in &players {
        finish_client_player(
            &mut commands,
            client_id.get_client_id(),
            &assets,
            entity,
            transform,
            player,
        );
    }
}

fn rotate_player(
    time: Res<Time>,
    mut player_query: Query<(&mut LastInput, &mut Transform), With<Player>>,
) {
    for (mut last_input, mut transform) in &mut player_query {
        // TODO: should the rate of change here be maxed?
        let delta_yaw = -last_input.input_state.look.x * time.delta_secs();

        transform.rotate_y(delta_yaw);

        last_input.input_state.look = Vec2::default();
    }
}

#[allow(clippy::type_complexity)]
fn update_player_physics(
    mut player_query: Query<(
        &mut LastInput,
        &mut TnuaController,
        &GlobalTransform,
        &mut PlayerPhysics,
    )>,
) {
    for (mut last_input, mut character_controller, global_transform, mut player_physics) in
        &mut player_query
    {
        // TODO: this needs lots of work to get working with avian

        let global_transform = global_transform.compute_transform();

        // handle move input
        let direction = global_transform.rotation
            * Vec3::new(
                last_input.input_state.r#move.x,
                0.0,
                -last_input.input_state.r#move.y,
            );
        // TODO: we may want to just max() each value instead of normalizing
        let direction = direction.normalize_or_zero();

        if direction.length_squared() > 0.0 {
            player_physics.velocity.x = direction.x * MOVE_SPEED;
            player_physics.velocity.z = direction.z * MOVE_SPEED;
        } else {
            player_physics.velocity.x = 0.0;
            player_physics.velocity.z = 0.0;
        }

        /*if last_input.jump {
            player_physics.velocity.y += JUMP_SPEED;
        }*/

        // apply gravity
        /*player_physics.velocity.y +=
            physics_config.single().gravity.y * gravity_scale.0 * time.delta_secs();
        player_physics.velocity.y = player_physics
            .velocity
            .y
            .clamp(-TERMINAL_VELOCITY, TERMINAL_VELOCITY);

        // move
        let translation = character_controller
            .translation
            .get_or_insert(Vec3::default());
        *translation += player_physics.velocity * time.delta_secs();*/
        character_controller.basis(TnuaBuiltinWalk {
            desired_velocity: player_physics.velocity,
            float_height: 1.5,
            ..Default::default()
        });

        if last_input.jump {
            character_controller.action(TnuaBuiltinJump {
                height: 4.0,
                ..Default::default()
            });
        }

        last_input.input_state.r#move = Vec2::default();
        last_input.jump = false;
    }
}
