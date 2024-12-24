use bevy::prelude::*;

use crate::OnInGame;

#[derive(Debug, Default, Component)]
#[require(Transform, Name(|| "Spawn Point"), OnInGame)]
pub struct SpawnPoint;
