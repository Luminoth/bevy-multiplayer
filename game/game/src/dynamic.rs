use avian3d::prelude::*;
use bevy::{color::palettes::css, prelude::*};
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{game::OnInGame, GameAssetState};

// TODO: if these moved to a resource
// they'd be easier to fudge for testing
const BALL_RADIUS: f32 = 0.5; //0.12;
const BALL_MASS: f32 = 0.5;
const BALL_RESTITUTION: f32 = 0.75;

#[derive(Debug, Component, Copy, Clone, Serialize, Deserialize)]
pub enum Dynamic {
    Ball,
}

impl Dynamic {
    pub fn get_name(&self) -> &'static str {
        match self {
            Dynamic::Ball => "Ball",
        }
    }
}

pub fn load_assets(
    meshes: &mut Assets<Mesh>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    game_assets: &mut GameAssetState,
) {
    game_assets.ball_mesh = meshes.add(Sphere::new(BALL_RADIUS));
    game_assets.ball_material = materials
        .as_mut()
        .map(|materials| materials.add(Color::from(css::FUCHSIA)))
        .unwrap_or_default();
}

pub fn spawn_ball(commands: &mut Commands, position: Vec3, assets: &GameAssetState) {
    info!("spawning ball at {} ...", position);

    let dynamic = Dynamic::Ball;
    let name = dynamic.get_name();

    commands.spawn((
        Mesh3d(assets.ball_mesh.clone()),
        MeshMaterial3d(assets.ball_material.clone()),
        Transform::from_xyz(position.x, position.y, position.z),
        RigidBody::Dynamic,
        // TODO: can we infer this from the mesh?
        Collider::sphere(BALL_RADIUS),
        Mass(BALL_MASS),
        Restitution::new(BALL_RESTITUTION),
        Name::new(name),
        Replicated,
        dynamic,
        OnInGame,
    ));
}

pub fn finish_client_dynamic(
    commands: &mut Commands,
    assets: &GameAssetState,
    entity: Entity,
    transform: &Transform,
    dynamic: &Dynamic,
) {
    info!(
        "finishing dynamic {} at {} ...",
        entity, transform.translation
    );

    commands.entity(entity).insert((
        Mesh3d(assets.ball_mesh.clone()),
        MeshMaterial3d(assets.ball_material.clone()),
        Name::new(format!("Replicated {}", dynamic.get_name())),
        OnInGame,
    ));
}

#[derive(Debug)]
pub struct DynamicPlugin;

impl Plugin for DynamicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            finish_client_dynamics
                .after(ClientSet::Receive)
                .run_if(client_connected),
        );

        app.replicate_group::<(Transform, Dynamic)>();
    }
}

#[allow(clippy::type_complexity)]
fn finish_client_dynamics(
    mut commands: Commands,
    assets: Option<Res<GameAssetState>>,
    dynamics: Query<(Entity, &Transform, &Dynamic), Without<Mesh3d>>,
) {
    let Some(assets) = assets else {
        return;
    };

    for (entity, transform, dynamic) in &dynamics {
        finish_client_dynamic(&mut commands, &assets, entity, transform, dynamic);
    }
}
