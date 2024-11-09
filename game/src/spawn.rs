use bevy::prelude::*;

use crate::OnInGame;

#[derive(Debug, Default, Component)]
pub struct SpawnPoint;

#[derive(Debug, Bundle)]
pub struct SpawnPointBundle {
    pub spawn_point: SpawnPoint,
    pub spatial: SpatialBundle,
    pub name: Name,
    pub tag: OnInGame,
}

impl SpawnPointBundle {
    #[allow(dead_code)]
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            spawn_point: SpawnPoint,
            spatial: SpatialBundle {
                transform: Transform::from_translation(translation),
                ..Default::default()
            },
            name: Name::new("Spawn Point"),
            tag: OnInGame,
        }
    }

    #[allow(dead_code)]
    pub fn from_transform(transform: Transform) -> Self {
        Self {
            spawn_point: SpawnPoint,
            spatial: SpatialBundle {
                transform,
                ..Default::default()
            },
            name: Name::new("Spawn Point"),
            tag: OnInGame,
        }
    }
}
