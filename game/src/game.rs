use bevy::{color::palettes::css::*, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::{ball, cleanup_state, player, world, GameState, InputState};

#[derive(Debug, Component)]
pub(crate) struct OnInGame;

#[derive(Debug, Resource)]
struct GameAssetState {
    floor_mesh: Handle<Mesh>,
    floor_material: Handle<StandardMaterial>,

    wall_material: Handle<StandardMaterial>,

    ball_mesh: Handle<Mesh>,
    ball_material: Handle<StandardMaterial>,

    player_mesh: Handle<Mesh>,
    player_material: Handle<StandardMaterial>,
}

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
        .add_systems(OnEnter(GameState::LoadAssets), load_assets)
        .add_systems(
            Update,
            wait_for_assets.run_if(in_state(GameState::LoadAssets)),
        )
        .add_systems(OnEnter(GameState::InGame), enter)
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
    game_state.set(GameState::InGame);
}

fn enter(mut commands: Commands, assets: Res<GameAssetState>) {
    info!("entering game ...");

    commands.insert_resource(ClearColor(Color::BLACK));

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

    // bouncing ball
    ball::spawn_ball(
        &mut commands,
        Vec3::new(0.0, 20.0, 0.0),
        assets.ball_mesh.clone(),
        assets.ball_material.clone(),
    );

    // player
    player::spawn_player(
        &mut commands,
        Vec3::new(-5.0, 2.1, 5.0),
        assets.player_mesh.clone(),
        assets.player_material.clone(),
    );
    player::spawn_camera(&mut commands, Vec3::new(0.0, 2.0, 15.0));
}

fn exit(mut commands: Commands) {
    info!("exiting game ...");

    commands.remove_resource::<ClearColor>();
    commands.remove_resource::<AmbientLight>();
    commands.remove_resource::<GameAssetState>();
}
