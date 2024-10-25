use bevy::prelude::*;

use crate::Settings;

#[derive(Debug)]
pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(bevy_replicon_renet::renet::RenetClient::new(
            bevy_replicon_renet::renet::ConnectionConfig::default(),
        ))
        .init_resource::<Settings>();
    }
}
