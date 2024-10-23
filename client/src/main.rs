mod camera;
mod client;
mod debug;
mod input;
mod main_menu;
mod ui;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

const DEFAULT_RESOLUTION: (f32, f32) = (1280.0, 720.0);

fn main() {
    println!("initializing client ...");

    let mut app = App::new();
    app.add_plugins((
        // bevy plugins
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
        // third-party plugins
        bevy_egui::EguiPlugin,
        bevy_mod_picking::DefaultPickingPlugins,
        RapierDebugRenderPlugin::default(),
        bevy_mod_reqwest::ReqwestPlugin::default(),
        // game plugins
        client::ClientPlugin,
        main_menu::MainMenuPlugin,
        camera::FpsCameraPlugin,
        input::InputPlugin,
        ui::UiPlugin,
        game::GamePlugin,
        debug::DebugPlugin,
    ))
    // update continuously even while unfocused (for networking)
    .insert_resource(bevy::winit::WinitSettings {
        focused_mode: bevy::winit::UpdateMode::Continuous,
        unfocused_mode: bevy::winit::UpdateMode::Continuous,
    })
    .init_state::<game::AppState>();

    info!("running client ...");
    app.run();
}