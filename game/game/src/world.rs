use std::borrow::Cow;

use avian3d::prelude::*;
use bevy::{color::palettes::css, prelude::*};

use crate::{GameAssetState, OnInGame};

// TODO: this should move to settings
const ENABLE_SHADOWS: bool = true;

pub fn load_assets(
    meshes: &mut Assets<Mesh>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    game_assets: &mut GameAssetState,
) {
    game_assets.floor_mesh = meshes.add(Plane3d::default().mesh().size(50.0, 50.0));
    game_assets.floor_material = materials
        .as_mut()
        .map(|materials| materials.add(Color::from(css::GREEN)))
        .unwrap_or_default();

    game_assets.wall_material = materials
        .as_mut()
        .map(|materials| materials.add(Color::from(css::NAVY)))
        .unwrap_or_default();
}

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
        RigidBody::Static,
        // TODO: can we infer this from the mesh?
        Collider::cuboid(50.0, 0.1, 50.0),
        Name::new(name),
        OnInGame,
    ));
}
