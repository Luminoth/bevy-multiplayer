use bevy::{color::palettes::css::*, prelude::*};
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Component)]
pub struct OnInGame;

#[derive(Debug, Component)]
pub struct MainCamera;

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Ball;

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Player;

#[derive(Debug, Resource)]
pub struct GameAssetState {
    floor_mesh: Handle<Mesh>,
    floor_material: Handle<StandardMaterial>,

    wall_material: Handle<StandardMaterial>,

    ball_mesh: Handle<Mesh>,
    ball_material: Handle<StandardMaterial>,

    player_mesh: Handle<Mesh>,
    player_material: Handle<StandardMaterial>,
}

pub fn load_assets(
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

pub fn wait_for_assets(mut game_state: ResMut<NextState<AppState>>) {
    game_state.set(AppState::InGame);
}

pub fn enter(mut commands: Commands, assets: Res<GameAssetState>) {
    info!("entering game ...");

    commands.insert_resource(ClearColor(Color::BLACK));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, 15.0),
            projection: PerspectiveProjection {
                fov: 90.0_f32.to_radians(),
                ..default()
            }
            .into(),
            ..default()
        },
        Name::new("Main Camera"),
        MainCamera,
        OnInGame,
    ));

    commands.insert_resource(AmbientLight {
        color: WHITE.into(),
        brightness: 80.0,
    });

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: ORANGE_RED.into(),
                shadows_enabled: true,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 5.0, 0.0),
                rotation: Quat::from_rotation_x(-45.0f32.to_radians()),
                ..default()
            },
            ..default()
        },
        Name::new("Sun"),
        OnInGame,
    ));

    // floor
    commands.spawn((
        MaterialMeshBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            mesh: assets.floor_mesh.clone(),
            material: assets.floor_material.clone(),
            ..default()
        },
        Collider::cuboid(25.0, 0.1, 25.0),
        Name::new("Ground"),
        OnInGame,
    ));

    // ceiling
    commands.spawn((
        MaterialMeshBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 50.0, 0.0),
                rotation: Quat::from_rotation_z(180.0f32.to_radians()),
                ..default()
            },
            mesh: assets.floor_mesh.clone(),
            material: assets.floor_material.clone(),
            ..default()
        },
        Collider::cuboid(25.0, 0.1, 25.0),
        Name::new("Ceiling"),
        OnInGame,
    ));

    // left wall
    commands.spawn((
        MaterialMeshBundle {
            transform: Transform {
                translation: Vec3::new(-25.0, 25.0, 0.0),
                rotation: Quat::from_rotation_z(-90.0f32.to_radians()),
                ..default()
            },
            mesh: assets.floor_mesh.clone(),
            material: assets.wall_material.clone(),
            ..default()
        },
        Collider::cuboid(25.0, 0.1, 25.0),
        Name::new("Left Wall"),
        OnInGame,
    ));

    // right wall
    commands.spawn((
        MaterialMeshBundle {
            transform: Transform {
                translation: Vec3::new(25.0, 25.0, 0.0),
                rotation: Quat::from_rotation_z(90.0f32.to_radians()),
                ..default()
            },
            mesh: assets.floor_mesh.clone(),
            material: assets.wall_material.clone(),
            ..default()
        },
        Collider::cuboid(25.0, 0.1, 25.0),
        Name::new("Right Wall"),
        OnInGame,
    ));

    // forward wall
    commands.spawn((
        MaterialMeshBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 25.0, -25.0),
                rotation: Quat::from_rotation_x(90.0f32.to_radians()),
                ..default()
            },
            mesh: assets.floor_mesh.clone(),
            material: assets.wall_material.clone(),
            ..default()
        },
        Collider::cuboid(25.0, 0.1, 25.0),
        Name::new("Forward Wall"),
        OnInGame,
    ));

    // rear wall
    commands.spawn((
        MaterialMeshBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 25.0, 25.0),
                rotation: Quat::from_rotation_x(-90.0f32.to_radians()),
                ..default()
            },
            mesh: assets.floor_mesh.clone(),
            material: assets.wall_material.clone(),
            ..default()
        },
        Collider::cuboid(25.0, 0.1, 25.0),
        Name::new("Rear Wall"),
        OnInGame,
    ));

    // bouncing ball
    commands.spawn((
        MaterialMeshBundle {
            transform: Transform::from_xyz(0.0, 20.0, 0.0),
            mesh: assets.ball_mesh.clone(),
            material: assets.ball_material.clone(),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::ball(0.5),
        ColliderMassProperties::Mass(0.5),
        Restitution::coefficient(0.7),
        Name::new("Ball"),
        Ball,
        OnInGame,
    ));

    // player
    commands.spawn((
        MaterialMeshBundle {
            transform: Transform::from_xyz(-5.0, 12.0, 5.0),
            mesh: assets.player_mesh.clone(),
            material: assets.player_material.clone(),
            ..default()
        },
        RigidBody::KinematicPositionBased,
        Velocity::default(),
        GravityScale(1.0),
        Collider::capsule_y(1.0, 1.0),
        ColliderMassProperties::Mass(100.0),
        KinematicCharacterController::default(),
        Name::new("Player"),
        Player,
        OnInGame,
    ));
}

pub fn exit(mut commands: Commands) {
    info!("exiting game ...");

    commands.remove_resource::<ClearColor>();
    commands.remove_resource::<AmbientLight>();
    commands.remove_resource::<GameAssetState>();
}

pub fn update_player_physics(
    physics_config: Res<RapierConfiguration>,
    time: Res<Time>,
    mut player_query: Query<
        (
            &mut KinematicCharacterController,
            &mut Velocity,
            &GravityScale,
        ),
        With<crate::game::Player>,
    >,
) {
    let (mut character_controller, mut velocity, gravity_scale) = player_query.single_mut();
    velocity.linvel = Vec3::new(0.0, physics_config.gravity.y * gravity_scale.0, 0.0);

    let translation = character_controller
        .translation
        .get_or_insert(Vec3::default());
    *translation += velocity.linvel * time.delta_seconds();
}
