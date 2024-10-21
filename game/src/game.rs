use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::AppState;

#[derive(Debug, Component)]
pub struct OnInGame;

#[derive(Debug, Resource)]
pub struct GameAssetState {
    floor_mesh: Handle<Mesh>,
    floor_material: Handle<StandardMaterial>,

    ball_mesh: Handle<Mesh>,
    ball_material: Handle<StandardMaterial>,
}

pub fn load_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_state: ResMut<NextState<AppState>>,
) {
    info!("loading assets ...");

    let floor_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(100.0)));
    let floor_material = materials.add(Color::srgb(0.0, 0.5, 0.0));

    let ball_mesh = meshes.add(Sphere::new(0.5));
    let ball_material = materials.add(Color::srgb(0.0, 0.5, 0.5));

    commands.insert_resource(GameAssetState {
        floor_mesh,
        floor_material,
        ball_mesh,
        ball_material,
    });

    game_state.set(AppState::InGame);
}

pub fn enter(mut commands: Commands, assets: Res<GameAssetState>) {
    info!("entering game ...");

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        Name::new("Main Camera"),
        OnInGame,
    ));

    // floor
    commands.spawn((
        MaterialMeshBundle {
            transform: Transform::from_xyz(0.0, -2.0, 0.0),
            mesh: assets.floor_mesh.clone(),
            material: assets.floor_material.clone(),
            ..default()
        },
        Collider::cuboid(100.0, 0.1, 100.0),
        Name::new("Ground"),
        OnInGame,
    ));

    // bouncing ball
    commands.spawn((
        MaterialMeshBundle {
            transform: Transform::from_xyz(0.0, 5.0, 0.0),
            mesh: assets.ball_mesh.clone(),
            material: assets.ball_material.clone(),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::ball(0.5),
        Restitution::coefficient(0.7),
        Name::new("Ball"),
        OnInGame,
    ));
}

pub fn exit(mut commands: Commands) {
    info!("exiting game ...");

    commands.remove_resource::<GameAssetState>();
}
