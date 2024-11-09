mod api;
mod camera;
mod client;
mod connect_server;
mod debug;
mod game;
mod game_menu;
mod input;
mod main_menu;
mod options;
mod settings;
mod ui;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;
use clap::Parser;

use options::Options;
use settings::Settings;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, States, Reflect)]
pub enum AppState {
    #[default]
    MainMenu,
    ConnectToServer,
    InGame,
}

const DEFAULT_RESOLUTION: (f32, f32) = (1280.0, 720.0);

fn main() {
    let options = Options::parse();

    println!("initializing client ...");

    let mut app = App::new();
    app
        // bevy plugins
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Multiplayer Jam".into(),
                        resolution: DEFAULT_RESOLUTION.into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    // default bevy filter plus silence some spammy 3rd party crates
                    filter: "wgpu=error,naga=warn,symphonia_core=error,symphonia_bundle_mp3=error"
                        .to_string(),
                    ..default()
                }),
        )
        // third-party plugins
        .add_plugins((
            bevy_egui::EguiPlugin,
            bevy_mod_picking::DefaultPickingPlugins,
            RapierDebugRenderPlugin::default(),
            RepliconPlugins,
            RepliconRenetPlugins,
            bevy_mod_reqwest::ReqwestPlugin::default(),
        ))
        // client / game plugins
        .add_plugins((
            client::ClientPlugin,
            game_common::GamePlugin,
            debug::DebugPlugin,
        ))
        // update continuously even while unfocused (for networking)
        .insert_resource(bevy::winit::WinitSettings {
            focused_mode: bevy::winit::UpdateMode::Continuous,
            unfocused_mode: bevy::winit::UpdateMode::Continuous,
        })
        .insert_resource(options)
        .init_state::<AppState>();

    info!("running client ...");
    app.run();
}
