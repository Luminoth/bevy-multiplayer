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
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;

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
fn init_app(app: &mut App) {
    println!("initializing client ...");

    app.add_plugins((
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
        bevy_egui::EguiPlugin,
        bevy_mod_picking::DefaultPickingPlugins,
        RapierDebugRenderPlugin::default(),
        client::ClientPlugin,
        camera::FpsCameraPlugin,
        debug::DebugPlugin,
    ))
    // update continuously even while unfocused (for networking)
    .insert_resource(bevy::winit::WinitSettings {
        focused_mode: bevy::winit::UpdateMode::Continuous,
        unfocused_mode: bevy::winit::UpdateMode::Continuous,
    })
    .init_resource::<input::InputState>()
    .add_event::<player::JumpEvent>()
    .add_systems(Update, ui::update_button)
    .add_systems(OnEnter(AppState::MainMenu), main_menu::enter)
    .add_systems(
        OnExit(AppState::MainMenu),
        (
            main_menu::exit,
            cleanup_state::<main_menu::OnMainMenu>,
            cleanup_state::<Node>,
        ),
    )
    .add_systems(
        Update,
        ((input::handle_gamepad_events, input::update_gamepad).chain())
            .run_if(in_state(AppState::InGame)),
    );

    app.register_type::<input::InputState>()
        .register_type::<player::PlayerPhysics>();
}

#[cfg(feature = "server")]
fn init_app(app: &mut App) {
    println!("initializing server ...");

    app.add_plugins((
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
    // rapier makes use of Mesh assets
    // and this is missing without rendering
    .init_asset::<Mesh>()
    .insert_resource(bevy_replicon_renet::renet::RenetServer::new(
        bevy_replicon_renet::renet::ConnectionConfig::default(),
    ))
    .add_systems(
        Update,
        server::wait_for_placement.run_if(in_state(AppState::WaitForPlacement)),
    )
    .add_systems(
        Update,
        server::init_network.run_if(in_state(AppState::InitServer)),
    )
    .add_systems(
        Update,
        server::handle_network_events.run_if(in_state(AppState::InGame)),
    )
    .add_systems(OnExit(AppState::InGame), server::shutdown_network);
}

fn main() {
    let mut app = App::new();
    init_app(&mut app);

    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default(),
        RepliconPlugins,
        RepliconRenetPlugins,
        bevy_mod_reqwest::ReqwestPlugin::default(),
        game::GamePlugin,
    ))
    .init_state::<AppState>()
    .add_systems(
        FixedUpdate,
        player::update_player_physics.run_if(in_state(AppState::InGame)),
    );

    // replication
    app.replicate_group::<(Transform, player::LocalPlayer)>();

    info!("running ...");
    app.run();
}
