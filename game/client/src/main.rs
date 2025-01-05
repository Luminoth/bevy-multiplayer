mod api;
mod camera;
mod client;
mod connect_server;
mod debug;
mod game;
mod game_menu;
mod input;
mod main_menu;
mod notifs;
mod options;
mod settings;
mod ui;

use bevy::{prelude::*, window::CursorGrabMode};
use bevy_rapier3d::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;
use bevy_tokio_tasks::TokioTasksPlugin;
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

fn recenter_cursor(window: Option<&mut Mut<'_, Window>>) {
    if let Some(window) = window {
        let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
        window.set_cursor_position(Some(center));
    }
}

pub fn show_cursor(window: Option<&mut Mut<'_, Window>>, show: bool) {
    if let Some(window) = window {
        window.cursor_options.grab_mode = if show {
            CursorGrabMode::None
        } else {
            CursorGrabMode::Locked
        };

        window.cursor_options.visible = show;
    }
}

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
                    filter: format!(
                        "{},symphonia_core=error,symphonia_bundle_mp3=error",
                        bevy::log::DEFAULT_FILTER
                    ),
                    ..default()
                }),
        )
        // third-party plugins
        .add_plugins((
            bevy_egui::EguiPlugin,
            RapierDebugRenderPlugin::default(),
            RepliconPlugins,
            RepliconRenetPlugins,
            /*bevy_replicon_snap::SnapshotInterpolationPlugin {
                max_tick_rate: game_common::SERVER_TICK_RATE,
            },*/
            TokioTasksPlugin::default(),
            bevy_mod_reqwest::ReqwestPlugin::default(),
            bevy_mod_websocket::WebSocketPlugin,
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
