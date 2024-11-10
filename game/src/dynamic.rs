use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{game::OnInGame, GameAssetState};

// TODO: make dynamic an enum or something so we don't need generics
// and give them a way to get their name and whatever else

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Dynamic;

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Ball;

#[derive(Bundle)]
pub struct ServerDynamicBundle<T>
where
    T: Component,
{
    material_mesh: MaterialMeshBundle<StandardMaterial>,
    rigidbody: RigidBody,
    collider: Collider,
    mass: ColliderMassProperties,
    restitution: Restitution,
    name: Name,
    replicated: Replicated,
    dynamic: T,
    tag: OnInGame,
}

#[derive(Bundle)]
pub struct ClientDynamicBundle {
    material_mesh: MaterialMeshBundle<StandardMaterial>,
    name: Name,
    tag: OnInGame,
}

pub fn spawn_dynamic<T>(
    commands: &mut Commands,
    position: Vec3,
    assets: &GameAssetState,
    dynamic: T,
) where
    T: Component,
{
    info!("spawning dynamic at {} ...", position);

    commands.spawn(ServerDynamicBundle {
        material_mesh: MaterialMeshBundle {
            transform: Transform::from_xyz(position.x, position.y, position.z),
            mesh: assets.ball_mesh.clone(),
            material: assets.ball_material.clone(),
            ..default()
        },
        rigidbody: RigidBody::Dynamic,
        collider: Collider::ball(0.5),
        mass: ColliderMassProperties::Mass(0.5),
        restitution: Restitution::coefficient(0.7),
        name: Name::new("Dynamic"),
        replicated: Replicated,
        dynamic,
        tag: OnInGame,
    });
}

pub fn finish_client_dynamic(
    commands: &mut Commands,
    assets: &GameAssetState,
    entity: Entity,
    transform: Transform,
) {
    info!(
        "finishing dynamic {} at {} ...",
        entity, transform.translation
    );

    commands.entity(entity).insert(ClientDynamicBundle {
        material_mesh: MaterialMeshBundle {
            transform,
            mesh: assets.ball_mesh.clone(),
            material: assets.ball_material.clone(),
            ..default()
        },
        name: Name::new("Replicated Dynamic"),
        tag: OnInGame,
    });
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
    dynamics: Query<(Entity, &Transform), (With<Dynamic>, Without<GlobalTransform>)>,
) {
    let Some(assets) = assets else {
        return;
    };

    for (entity, transform) in &dynamics {
        finish_client_dynamic(&mut commands, &assets, entity, *transform);
    }
}
