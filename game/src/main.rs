mod ball;
mod camera;
mod client;
mod debug;
mod game;
mod input;
mod main_menu;
mod player;
mod server;
mod ui;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[cfg(not(feature = "server"))]
const DEFAULT_RESOLUTION: (f32, f32) = (1280.0, 720.0);

// TODO: this sets the server "frame rate"
// bevy FixedUpdate tho runs at 64hz
// and that might need to be adjusted as well?
#[cfg(feature = "server")]
const SERVER_TICK_RATE: f64 = 1.0 / 60.0;

pub const PROTOCOL_ID: u64 = 0;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, States, Reflect)]
pub enum AppState {
    #[cfg(not(feature = "server"))]
    #[default]
    MainMenu,

    #[cfg(not(feature = "server"))]
    ConnectToServer,

    #[cfg(not(feature = "server"))]
    WaitForConnect,

    #[cfg(feature = "server")]
    #[default]
    WaitForPlacement,

    #[cfg(feature = "server")]
    InitServer,

    LoadAssets,
    InGame,
}

pub fn cleanup_state<T>(mut commands: Commands, query: Query<Entity, With<T>>)
where
    T: Component,
{
    for e in &query {
        commands.entity(e).despawn_recursive();
    }
}

#[cfg(not(feature = "server"))]
fn run() {
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
    .init_state::<AppState>();

    info!("running client ...");
    app.run();
}

#[cfg(feature = "server")]
fn run() {
    println!("initializing server ...");

    let mut app = App::new();
    app.add_plugins((
        // bevy plugins
        // TODO: replace with HeadlessPlugins in 0.15
        // (it includes all the plugins that Minimal is missing)
        MinimalPlugins.set(bevy::app::ScheduleRunnerPlugin::run_loop(
            bevy::utils::Duration::from_secs_f64(SERVER_TICK_RATE),
        )),
        bevy::app::PanicHandlerPlugin,
        bevy::log::LogPlugin::default(),
        bevy::transform::TransformPlugin,
        bevy::hierarchy::HierarchyPlugin,
        bevy::diagnostic::DiagnosticsPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin,
        bevy::animation::AnimationPlugin,
        bevy::state::app::StatesPlugin,
        // third-party plugins
        bevy_mod_reqwest::ReqwestPlugin::default(),
        // game plugins
        server::ServerPlugin,
        game::GamePlugin,
    ))
    .init_state::<AppState>();

    info!("running server ...");
    app.run();
}

fn main() {
    run();
}
