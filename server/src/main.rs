mod api;
mod game;
mod options;
mod orchestration;
mod placement;
mod server;
mod tasks;
mod websocket;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;
use bevy_tokio_tasks::TokioTasksPlugin;
use clap::Parser;

use common::gameserver::GameServerState;
use game_common::SERVER_TICK_RATE;

use options::Options;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, States, Reflect)]
pub enum AppState {
    #[default]
    Startup,
    WaitForPlacement,
    InitServer,
    InGame,
    Shutdown,
}

impl From<AppState> for GameServerState {
    fn from(state: AppState) -> Self {
        match state {
            AppState::Startup => GameServerState::Init,
            AppState::WaitForPlacement => GameServerState::WaitingForPlacement,
            AppState::InitServer => GameServerState::Loading,
            AppState::InGame => GameServerState::InGame,
            AppState::Shutdown => GameServerState::Shutdown,
        }
    }
}

impl AppState {
    #[inline]
    pub fn is_ready(&self) -> bool {
        *self != AppState::Startup
    }
}

pub fn is_not_headless(options: Res<Options>) -> bool {
    !options.headless
}

fn main() {
    let options = Options::parse();

    println!("initializing server ...");

    let mut app = App::new();

    // bevy plugins
    if options.headless {
        app.add_plugins((
            // TODO: replace with HeadlessPlugins in 0.15
            // (it includes all the plugins that Minimal is missing)
            MinimalPlugins.set(bevy::app::ScheduleRunnerPlugin::run_loop(
                bevy::utils::Duration::from_secs_f64(1.0 / SERVER_TICK_RATE as f64),
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
        ));

        // rapier makes use of Mesh assets
        // and this is missing without rendering
        app.init_asset::<Mesh>();
    } else {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Multiplayer Jam - Server".into(),
                        resolution: (1280.0, 720.0).into(),
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
        );
    }

    app
        // bevy plugins
        // third-party plugins
        .add_plugins((
            RepliconPlugins, /*.set(ServerPlugin {
                                 tick_policy: TickPolicy::MaxTickRate(SERVER_TICK_RATE),
                                 ..default()
                             })*/
            RepliconRenetPlugins,
            bevy_mod_reqwest::ReqwestPlugin::default(),
            TokioTasksPlugin::default(),
        ))
        // server / game plugins
        .add_plugins((server::ServerPlugin, game_common::GamePlugin))
        .insert_resource(options)
        .init_state::<AppState>();

    info!("running server ...");
    app.run();
}
