use avian3d::prelude::*;
use bevy::{color::palettes::css, ecs::entity::MapEntities, prelude::*};
use bevy_replicon::prelude::*;
use bevy_tnua::{
    TnuaAnimatingState, TnuaAnimatingStateDirective, builtins::TnuaBuiltinJumpState, prelude::*,
};
use serde::{Deserialize, Serialize};

use common::user::UserId;

use crate::{GameAssetState, GameState, InputState, game::OnInGame, network::PlayerClientId};

// TODO: if these moved to a resource
// they'd be easier to fudge for testing
const MOVE_SPEED: f32 = 10.0;
//const JUMP_SPEED: f32 = 15.0;
const JUMP_HEIGHT: f32 = 8.0;
//const TERMINAL_VELOCITY: f32 = 50.0;
const HEIGHT: f32 = 2.0; // includes capsule hemispheres
const MASS: f32 = 75.0;

#[derive(Debug, strum::Display)]
pub enum PlayerAnimationState {
    Idle,
    Walking,
    Running(f32),
    Jumping,
    Falling,
    // TODO: this should probably be split crouching / crouched / standing
    Crouching,
}

#[derive(Debug, Default, Resource)]
pub struct PlayerAnimationAssets {
    graph: Handle<AnimationGraph>,

    idle_animation_index: AnimationNodeIndex,
    walk_animation_index: AnimationNodeIndex,
    run_animation_index: AnimationNodeIndex,
    jump_animation_index: AnimationNodeIndex,
    fall_animation_index: AnimationNodeIndex,
    crouch_animation_index: AnimationNodeIndex,
}

#[derive(Debug, Copy, Clone, Component, Reflect, Serialize, Deserialize)]
pub struct Player {
    pub user_id: UserId,
    pub client_id: ClientId,

    pub crouched: bool,
}

impl Player {
    fn new(user_id: UserId, client_id: ClientId) -> Self {
        Self {
            user_id,
            client_id,
            crouched: false,
        }
    }
}

#[derive(Debug, Component)]
pub struct LocalPlayer;

#[derive(Debug, Component)]
pub struct PlayerCamera;

#[derive(Debug, Default, Component)]
pub struct LastInput {
    pub input_state: InputState,
    pub jump: bool,
}

#[derive(Debug, Event, Serialize, Deserialize)]
pub struct PlayerCrouchEvent(pub Entity, pub bool);

impl MapEntities for PlayerCrouchEvent {
    fn map_entities<T: EntityMapper>(&mut self, entity_mapper: &mut T) {
        self.0 = entity_mapper.map_entity(self.0);
    }
}

pub fn load_assets(
    meshes: &mut Assets<Mesh>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    animations: &mut ResMut<Assets<AnimationClip>>,
    graphs: &mut ResMut<Assets<AnimationGraph>>,
    game_assets: &mut GameAssetState,
) {
    game_assets.player_mesh = meshes.add(Capsule3d::new(HEIGHT * 0.5, HEIGHT));
    game_assets.player_material = materials
        .as_mut()
        .map(|materials| materials.add(Color::from(css::LIGHT_YELLOW)))
        .unwrap_or_default();

    create_animations(animations, graphs, game_assets);
}

fn create_animations(
    animations: &mut ResMut<Assets<AnimationClip>>,
    graphs: &mut ResMut<Assets<AnimationGraph>>,
    game_assets: &mut GameAssetState,
) {
    let mut graph = AnimationGraph::new();

    let idle_animation = AnimationClip::default();
    // TODO: create the animation

    game_assets.player_animations.idle_animation_index =
        graph.add_clip(animations.add(idle_animation), 1.0, graph.root);

    let walk_animation = AnimationClip::default();
    // TODO: create the animation

    game_assets.player_animations.walk_animation_index =
        graph.add_clip(animations.add(walk_animation), 1.0, graph.root);

    let run_animation = AnimationClip::default();
    // TODO: create the animation

    game_assets.player_animations.run_animation_index =
        graph.add_clip(animations.add(run_animation), 1.0, graph.root);

    let jump_animation = AnimationClip::default();
    // TODO: create the animation

    game_assets.player_animations.jump_animation_index =
        graph.add_clip(animations.add(jump_animation), 1.0, graph.root);

    let fall_animation = AnimationClip::default();
    // TODO: create the animation

    game_assets.player_animations.fall_animation_index =
        graph.add_clip(animations.add(fall_animation), 1.0, graph.root);

    let crouch_animation = AnimationClip::default();
    // TODO: create the animation

    game_assets.player_animations.crouch_animation_index =
        graph.add_clip(animations.add(crouch_animation), 1.0, graph.root);

    game_assets.player_animations.graph = graphs.add(graph);
}

pub fn spawn_player(
    commands: &mut Commands,
    user_id: UserId,
    client_id: ClientId,
    position: Vec3,
    assets: &GameAssetState,
) -> Entity {
    info!("spawning player {:?} at {} ...", user_id, position);

    let mut commands = commands.spawn((
        Mesh3d(assets.player_mesh.clone()),
        MeshMaterial3d(assets.player_material.clone()),
        AnimationGraphHandle(assets.player_animations.graph.clone()),
        AnimationPlayer::default(),
        Transform::from_xyz(position.x, position.y, position.z),
        Name::new(format!("Player {}: {:?}", user_id, client_id)),
        Replicated,
        LastInput::default(),
        Player::new(user_id, client_id),
        OnInGame,
    ));

    commands.insert((
        RigidBody::Dynamic,
        // TODO: can we infer this from the mesh?
        Collider::capsule(HEIGHT * 0.5, HEIGHT),
        Mass(MASS),
        LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
    ));

    commands
        .insert((
            TnuaController::default(),
            TnuaAnimatingState::<PlayerAnimationState>::default(),
            bevy_tnua_avian3d::TnuaAvian3dSensorShape(Collider::cylinder(0.5, 0.0)),
        ))
        .id()
}

pub fn despawn_player(commands: &mut Commands, entity: Entity, user_id: UserId) {
    info!("despawning player {} ...", user_id);

    commands.entity(entity).despawn_recursive();
}

pub fn finish_client_player(
    commands: &mut Commands,
    local_client_id: ClientId,
    assets: &GameAssetState,
    entity: Entity,
    transform: &Transform,
    player: &Player,
) {
    info!(
        "finishing player {} ({:?}:{:?}) at {} ...",
        player.user_id, player.client_id, local_client_id, transform.translation
    );

    let is_local = player.client_id == local_client_id;

    let mut ec = commands.entity(entity);
    ec.insert((
        Mesh3d(assets.player_mesh.clone()),
        MeshMaterial3d(assets.player_material.clone()),
        AnimationGraphHandle(assets.player_animations.graph.clone()),
        AnimationPlayer::default(),
        Collider::capsule(HEIGHT * 0.5, HEIGHT),
        Name::new(format!(
            "Replicated Player ({}) {}: {:?}",
            player.user_id,
            if is_local { " Local" } else { "Remote" },
            player.client_id
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

    // spawn the player camera
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
        .add_systems(
            Update,
            (rotate_player, animate_player)
                .run_if(server_or_singleplayer)
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            FixedUpdate,
            update_player_physics
                .run_if(server_or_singleplayer)
                .run_if(in_state(GameState::InGame)),
        );

        app.register_type::<Player>();

        app.add_mapped_server_event::<PlayerCrouchEvent>(ChannelKind::Ordered)
            .replicate_group::<(Transform, Player)>();
    }
}

fn finish_client_players(
    mut commands: Commands,
    client_id: Res<PlayerClientId>,
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

fn animate_player(
    game_assets: Res<GameAssetState>,
    mut player_query: Query<(
        &TnuaController,
        &mut TnuaAnimatingState<PlayerAnimationState>,
        &mut AnimationPlayer,
    )>,
) {
    for (controller, mut animating_state, mut animation_player) in &mut player_query {
        // get the current action the player is performing
        // TODO: handle crouching
        let current_status_for_animating = match controller.action_name() {
            Some(TnuaBuiltinJump::NAME) => {
                let (_, jump_state) = controller.concrete_action::<TnuaBuiltinJump>().unwrap();
                match jump_state {
                    TnuaBuiltinJumpState::NoJump => PlayerAnimationState::Idle,
                    TnuaBuiltinJumpState::FallSection => PlayerAnimationState::Falling,
                    _ => PlayerAnimationState::Jumping,
                }
            }
            Some(other) => {
                error!("Unknown tnua action: {}", other);
                PlayerAnimationState::Idle
            }
            None => {
                if let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>() {
                    if basis_state.standing_on_entity().is_none() {
                        PlayerAnimationState::Falling
                    } else {
                        // TODO: walk / run based on speed
                        let speed = basis_state.running_velocity.length();
                        if 0.01 < speed {
                            PlayerAnimationState::Running(0.1 * speed)
                        } else {
                            PlayerAnimationState::Idle
                        }
                    }
                } else {
                    PlayerAnimationState::Idle
                }
            }
        };

        // update the animation state
        let animating_directive =
            animating_state.update_by_discriminant(current_status_for_animating);
        match animating_directive {
            TnuaAnimatingStateDirective::Maintain { state } => {
                if let PlayerAnimationState::Running(speed) = state {
                    // TODO: walk / run based on speed
                    if let Some(animation) = animation_player
                        .animation_mut(game_assets.player_animations.run_animation_index)
                    {
                        animation.set_speed(*speed);
                    }
                }
            }
            TnuaAnimatingStateDirective::Alter {
                old_state: _,
                state,
            } => {
                // TODO: transition animations
                animation_player.stop_all();

                info!("animation update: {}", state);
                match state {
                    PlayerAnimationState::Idle => {
                        animation_player
                            .start(game_assets.player_animations.idle_animation_index)
                            .set_speed(1.0)
                            .repeat();
                    }
                    PlayerAnimationState::Walking => {
                        animation_player
                            .start(game_assets.player_animations.walk_animation_index)
                            .repeat();
                    }
                    PlayerAnimationState::Running(speed) => {
                        animation_player
                            .start(game_assets.player_animations.run_animation_index)
                            .set_speed(*speed)
                            .repeat();
                    }
                    PlayerAnimationState::Jumping => {
                        animation_player
                            .start(game_assets.player_animations.jump_animation_index)
                            .set_speed(2.0);
                    }
                    PlayerAnimationState::Falling => {
                        animation_player
                            .start(game_assets.player_animations.fall_animation_index)
                            .set_speed(1.0);
                    }
                    PlayerAnimationState::Crouching => {
                        animation_player
                            .start(game_assets.player_animations.crouch_animation_index)
                            .set_speed(1.0);
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn update_player_physics(
    mut player_query: Query<(
        Entity,
        &mut Player,
        &mut LastInput,
        &mut TnuaController,
        &GlobalTransform,
    )>,
    mut evw_crouch: EventWriter<ToClients<PlayerCrouchEvent>>,
) {
    for (entity, mut player, mut last_input, mut character_controller, global_transform) in
        &mut player_query
    {
        let global_transform = global_transform.compute_transform();

        let direction = global_transform.rotation
            * Vec3::new(
                last_input.input_state.r#move.x,
                0.0,
                -last_input.input_state.r#move.y,
            );
        last_input.input_state.r#move = Vec2::default();

        character_controller.basis(TnuaBuiltinWalk {
            desired_velocity: direction.normalize_or_zero() * MOVE_SPEED,
            // TODO: this isn't right, but we should probably do this instead of rotate_player()
            //desired_forward: Dir3::new(Vec3::new(last_input.input_state.look.x, 0.0, 0.0)).ok(),
            // TODO: this doesn't seem right by the docs, but anything less doesn't work
            float_height: HEIGHT,
            ..Default::default()
        });

        if last_input.jump {
            character_controller.action(TnuaBuiltinJump {
                height: JUMP_HEIGHT,
                ..Default::default()
            });
            last_input.jump = false;
        }

        // TODO: this is looking more like it should be an event
        // (start crouch, end crouch)
        // TODO: client needs a reader next, that should "crouch" the player
        // TODO: try and figure out how to use the tnua crouch action?
        if last_input.input_state.crouch {
            if !player.crouched {
                player.crouched = true;
                evw_crouch.send(ToClients {
                    mode: SendMode::Broadcast,
                    event: PlayerCrouchEvent(entity, true),
                });
            }
        } else if player.crouched {
            player.crouched = false;
            evw_crouch.send(ToClients {
                mode: SendMode::Broadcast,
                event: PlayerCrouchEvent(entity, false),
            });
        }
    }
}
