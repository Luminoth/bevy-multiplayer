use std::borrow::Cow;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::OnInGame;

const ENABLE_SHADOWS: bool = true;

pub fn spawn_directional_light(
    commands: &mut Commands,
    color: Color,
    transform: Transform,
    name: impl Into<Cow<'static, str>>,
) {
    commands.spawn((
        DirectionalLight {
            color,
            shadows_enabled: ENABLE_SHADOWS,
            ..default()
        },
        transform,
        Name::new(name),
        OnInGame,
    ));
}

pub fn spawn_wall(
    commands: &mut Commands,
    transform: Transform,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    name: impl Into<Cow<'static, str>>,
) {
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        transform,
        Collider::cuboid(50.0, 0.1, 50.0),
        Name::new(name),
        OnInGame,
    ));
}
