use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{game::OnInGame, GameAssetState};

// TODO: generalize this, instead of just the bouncing ball
// make it handle any server-authoritative dynamic thing

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Ball;

#[derive(Bundle)]
pub struct ServerBallBundle {
    material_mesh: MaterialMeshBundle<StandardMaterial>,
    rigidbody: RigidBody,
    collider: Collider,
    mass: ColliderMassProperties,
    restitution: Restitution,
    name: Name,
    replicated: Replicated,
    ball: Ball,
    tag: OnInGame,
}

#[derive(Bundle)]
pub struct ClientBallBundle {
    material_mesh: MaterialMeshBundle<StandardMaterial>,
    name: Name,
    tag: OnInGame,
}

pub fn spawn_ball(commands: &mut Commands, position: Vec3, assets: &GameAssetState) {
    info!("spawning ball at {} ...", position);

    commands.spawn(ServerBallBundle {
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
        name: Name::new("Ball"),
        replicated: Replicated,
        ball: Ball,
        tag: OnInGame,
    });
}

pub fn finish_client_ball(
    commands: &mut Commands,
    assets: &GameAssetState,
    entity: Entity,
    transform: Transform,
) {
    info!("finishing ball {} at {} ...", entity, transform.translation);

    commands.entity(entity).insert(ClientBallBundle {
        material_mesh: MaterialMeshBundle {
            transform,
            mesh: assets.ball_mesh.clone(),
            material: assets.ball_material.clone(),
            ..default()
        },
        name: Name::new("Replicated Ball"),
        tag: OnInGame,
    });
}

#[derive(Debug)]
pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            finish_client_balls
                .after(ClientSet::Receive)
                .run_if(client_connected),
        );

        app.replicate_group::<(Transform, Ball)>();
    }
}

#[allow(clippy::type_complexity)]
fn finish_client_balls(
    mut commands: Commands,
    assets: Option<Res<GameAssetState>>,
    balls: Query<(Entity, &Transform), (With<Ball>, Without<GlobalTransform>)>,
) {
    let Some(assets) = assets else {
        return;
    };

    for (entity, transform) in &balls {
        finish_client_ball(&mut commands, &assets, entity, *transform);
    }
}
