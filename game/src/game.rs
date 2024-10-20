use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Debug, Component)]
pub struct OnInGame;

pub fn enter(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        Name::new("Main Camera"),
        OnInGame,
    ));

    // ground
    commands.spawn((
        Collider::cuboid(100.0, 0.1, 100.0),
        TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)),
        Name::new("Ground"),
        OnInGame,
    ));

    // bouncing ball
    commands.spawn((
        RigidBody::Dynamic,
        Collider::ball(0.5),
        Restitution::coefficient(0.7),
        TransformBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)),
        Name::new("Ball"),
        OnInGame,
    ));
}
