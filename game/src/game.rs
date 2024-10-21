use bevy::{color::palettes::css::*, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::AppState;

#[derive(Debug, Component)]
pub struct OnInGame;

#[derive(Debug, Component)]
pub struct Ball;

#[derive(Debug, Resource)]
pub struct GameAssetState {
    floor_mesh: Handle<Mesh>,
    floor_material: Handle<StandardMaterial>,

    wall_material: Handle<StandardMaterial>,

    ball_mesh: Handle<Mesh>,
    ball_material: Handle<StandardMaterial>,
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
        .and_then(|materials| Some(materials.add(Color::from(GREEN))));

    let wall_material = materials
        .as_mut()
        .and_then(|materials| Some(materials.add(Color::from(NAVY))));

    let ball_mesh = meshes.add(Sphere::new(0.5));
    let ball_material = materials
        .as_mut()
        .and_then(|materials| Some(materials.add(Color::from(FUCHSIA))));

    commands.insert_resource(GameAssetState {
        floor_mesh,
        floor_material: floor_material.unwrap_or_default(),
        wall_material: wall_material.unwrap_or_default(),
        ball_mesh,
        ball_material: ball_material.unwrap_or_default(),
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
            ..default()
        },
        Name::new("Main Camera"),
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

    // reard wall
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
            transform: Transform::from_xyz(0.0, 25.0, 0.0),
            mesh: assets.ball_mesh.clone(),
            material: assets.ball_material.clone(),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::ball(0.5),
        Restitution::coefficient(0.7),
        Name::new("Ball"),
        Ball,
        OnInGame,
    ));
}

pub fn update(_ball_query: Query<&Transform, With<Ball>>) {
    /*for ball in &_ball_query {
        info!("ball position: {}", ball.translation);
    }*/
}

pub fn exit(mut commands: Commands) {
    info!("exiting game ...");

    commands.remove_resource::<ClearColor>();
    commands.remove_resource::<AmbientLight>();
    commands.remove_resource::<GameAssetState>();
}
