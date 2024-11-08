use bevy::{color::palettes::css::*, prelude::*};
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;

use crate::{
    ball, cleanup_state,
    network::{InputUpdateEvent, PlayerClientId, PlayerJumpEvent},
    player, spawn, world, GameAssetState, GameState, InputState,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct ServerSet;

#[derive(Debug, Component)]
pub struct OnInGame;

#[derive(Debug)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // third-party plugins
            RapierPhysicsPlugin::<NoUserData>::default(),
            // game plugins
            player::PlayerPlugin,
            ball::BallPlugin,
        ))
        .init_state::<GameState>()
        .init_resource::<InputState>()
        // TOOD: move to a network plugin
        .add_client_event::<InputUpdateEvent>(ChannelKind::Ordered)
        .add_client_event::<PlayerJumpEvent>(ChannelKind::Unordered)
        .add_systems(OnEnter(GameState::LoadAssets), load_assets)
        .add_systems(
            Update,
            wait_for_assets.run_if(in_state(GameState::LoadAssets)),
        )
        .add_systems(OnEnter(GameState::SpawnWorld), spawn_world)
        .add_systems(
            Update,
            wait_for_world.run_if(in_state(GameState::SpawnWorld)),
        )
        .add_systems(
            OnEnter(GameState::InGame),
            (
                enter_server
                    .run_if(server_or_singleplayer)
                    .in_set(ServerSet),
                enter_client.run_if(client_connected),
            ),
        )
        .add_systems(
            OnExit(GameState::InGame),
            (exit, cleanup_state::<OnInGame>, cleanup_state::<Node>),
        );

        app.register_type::<InputState>();
    }
}

fn load_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    info!("loading assets ...");

    let floor_mesh = meshes.add(Plane3d::default().mesh().size(50.0, 50.0));
    let floor_material = materials
        .as_mut()
        .map(|materials| materials.add(Color::from(GREEN)));

    let wall_material = materials
        .as_mut()
        .map(|materials| materials.add(Color::from(NAVY)));

    let ball_mesh = meshes.add(Sphere::new(0.5));
    let ball_material = materials
        .as_mut()
        .map(|materials| materials.add(Color::from(FUCHSIA)));

    let player_mesh = meshes.add(Capsule3d::new(1.0, 2.0));
    let player_material = materials
        .as_mut()
        .map(|materials| materials.add(Color::from(LIGHT_YELLOW)));

    commands.insert_resource(GameAssetState {
        floor_mesh,
        floor_material: floor_material.unwrap_or_default(),
        wall_material: wall_material.unwrap_or_default(),
        ball_mesh,
        ball_material: ball_material.unwrap_or_default(),
        player_mesh,
        player_material: player_material.unwrap_or_default(),
    });
}

fn wait_for_assets(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::SpawnWorld);
}

// TODO: it would be nice if we could not load materials on the server

fn spawn_world(mut commands: Commands, assets: Res<GameAssetState>) {
    info!("spawning world ...");

    commands.insert_resource(AmbientLight {
        color: WHITE.into(),
        brightness: 80.0,
    });

    world::spawn_directional_light(
        &mut commands,
        ORANGE_RED.into(),
        Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            rotation: Quat::from_rotation_x(-45.0f32.to_radians()),
            ..default()
        },
        "Sun",
    );

    // floor
    world::spawn_wall(
        &mut commands,
        Transform::from_xyz(0.0, 0.0, 0.0),
        assets.floor_mesh.clone(),
        assets.floor_material.clone(),
        "Ground",
    );

    // ceiling
    world::spawn_wall(
        &mut commands,
        Transform {
            translation: Vec3::new(0.0, 50.0, 0.0),
            rotation: Quat::from_rotation_z(180.0f32.to_radians()),
            ..default()
        },
        assets.floor_mesh.clone(),
        assets.floor_material.clone(),
        "Ceiling",
    );

    // left wall
    world::spawn_wall(
        &mut commands,
        Transform {
            translation: Vec3::new(-25.0, 25.0, 0.0),
            rotation: Quat::from_rotation_z(-90.0f32.to_radians()),
            ..default()
        },
        assets.floor_mesh.clone(),
        assets.wall_material.clone(),
        "Left Wall",
    );

    // right wall
    world::spawn_wall(
        &mut commands,
        Transform {
            translation: Vec3::new(25.0, 25.0, 0.0),
            rotation: Quat::from_rotation_z(90.0f32.to_radians()),
            ..default()
        },
        assets.floor_mesh.clone(),
        assets.wall_material.clone(),
        "Right Wall",
    );

    // forward wall
    world::spawn_wall(
        &mut commands,
        Transform {
            translation: Vec3::new(0.0, 25.0, -25.0),
            rotation: Quat::from_rotation_x(90.0f32.to_radians()),
            ..default()
        },
        assets.floor_mesh.clone(),
        assets.wall_material.clone(),
        "Forward Wall",
    );

    // rear wall
    world::spawn_wall(
        &mut commands,
        Transform {
            translation: Vec3::new(0.0, 25.0, 25.0),
            rotation: Quat::from_rotation_x(-90.0f32.to_radians()),
            ..default()
        },
        assets.floor_mesh.clone(),
        assets.wall_material.clone(),
        "Rear Wall",
    );

    // player spawns
    commands.spawn(spawn::SpawnPointBundle::from_translation(Vec3::new(
        -5.0, 2.1, 5.0,
    )));
}

fn wait_for_world(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::InGame);
}

fn enter_server(mut commands: Commands, assets: Res<GameAssetState>) {
    info!("entering server game ...");

    ball::spawn_ball(&mut commands, Vec3::new(0.0, 20.0, -5.0), &assets);
}

#[allow(clippy::type_complexity)]
pub fn spawn_client_world(
    commands: &mut Commands,
    client_id: ClientId,
    assets: &GameAssetState,
    balls: &Query<(Entity, &Transform), (With<ball::Ball>, Without<GlobalTransform>)>,
    players: &Query<(Entity, &Transform, &player::Player), Without<GlobalTransform>>,
) {
    info!("spawning client world ...");

    commands.insert_resource(ClearColor(Color::BLACK));

    for (entity, transform) in balls {
        ball::finish_client_ball(commands, assets, entity, *transform);
    }

    for (entity, transform, player) in players {
        player::finish_client_player(commands, client_id, assets, entity, *transform, *player);
    }
}

#[allow(clippy::type_complexity)]
fn enter_client(
    mut commands: Commands,
    client_id: Res<PlayerClientId>,
    assets: Res<GameAssetState>,
    balls: Query<(Entity, &Transform), (With<ball::Ball>, Without<GlobalTransform>)>,
    players: Query<(Entity, &Transform, &player::Player), Without<GlobalTransform>>,
) {
    info!("entering client game ...");

    spawn_client_world(
        &mut commands,
        client_id.get_client_id(),
        &assets,
        &balls,
        &players,
    );
}

fn exit(mut commands: Commands) {
    info!("exiting game ...");

    commands.remove_resource::<ClearColor>();
    commands.remove_resource::<AmbientLight>();
    commands.remove_resource::<GameAssetState>();
}
