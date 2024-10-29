mod api;
mod options;
mod orchestration;
mod placement;
mod server;
mod tasks;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;
use bevy_tokio_tasks::TokioTasksPlugin;
use clap::Parser;

use options::Options;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, States, Reflect)]
pub enum AppState {
    #[default]
    Startup,
    WaitForPlacement,
    InitServer,
    LoadAssets,
    InGame,
}

// TODO: this sets the server "frame rate"
// bevy FixedUpdate tho runs at 64hz
// and that might need to be adjusted as well?
const SERVER_TICK_RATE: f64 = 1.0 / 60.0;

fn main() {
    let options = Options::parse();

    println!("initializing server ...");

    let mut app = App::new();
    app
        // bevy plugins
        .add_plugins((
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
        ))
        // third-party plugins
        .add_plugins((
            RepliconPlugins,
            RepliconRenetPlugins,
            bevy_mod_reqwest::ReqwestPlugin::default(),
            TokioTasksPlugin::default(),
        ))
        // server / game plugins
        .add_plugins((server::ServerPlugin, game::GamePlugin))
        .insert_resource(options)
        .init_state::<AppState>();

    info!("running server ...");
    app.run();
}
