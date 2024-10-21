mod game;
mod main_menu;
mod server;
mod ui;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[cfg(not(feature = "server"))]
const DEFAULT_RESOLUTION: (f32, f32) = (1280.0, 720.0);

#[cfg(feature = "server")]
const SERVER_TICK_RATE: f64 = 1.0 / 60.0;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, States)]
pub enum AppState {
    #[cfg(not(feature = "server"))]
    #[default]
    MainMenu,

    #[cfg(feature = "server")]
    #[default]
    WaitingForPlacement,

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
        bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
        bevy_mod_picking::DefaultPickingPlugins,
        RapierDebugRenderPlugin::default(),
    ))
    .add_systems(OnEnter(AppState::MainMenu), main_menu::enter)
    .add_systems(
        OnExit(AppState::MainMenu),
        (
            cleanup_state::<main_menu::OnMainMenu>,
            cleanup_state::<Node>,
        ),
    );
}

#[cfg(feature = "server")]
fn init_app(app: &mut App) {
    println!("initializing server ...");

    app.add_plugins((
        MinimalPlugins.set(bevy::app::ScheduleRunnerPlugin::run_loop(
            bevy::utils::Duration::from_secs_f64(SERVER_TICK_RATE),
        )),
    ))
    .add_systems(
        Update,
        server::wait_for_placement.run_if(in_state(AppState::WaitingForPlacement)),
    );
}

fn main() {
    let mut app = App::new();
    init_app(&mut app);

    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default(),
        bevy_mod_reqwest::ReqwestPlugin::default(),
    ))
    .init_state::<AppState>()
    .add_systems(OnEnter(AppState::LoadAssets), game::load_assets)
    .add_systems(OnEnter(AppState::InGame), game::enter)
    .add_systems(
        OnExit(AppState::InGame),
        (
            game::exit,
            cleanup_state::<game::OnInGame>,
            cleanup_state::<Node>,
        ),
    );

    info!("running ...");
    app.run();
}
