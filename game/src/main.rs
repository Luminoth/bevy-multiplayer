use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[cfg(not(feature = "server"))]
const DEFAULT_RESOLUTION: (f32, f32) = (1280.0, 720.0);

#[cfg(feature = "server")]
const SERVER_TICK_RATE: f64 = 1.0 / 60.0;

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
        RapierDebugRenderPlugin::default(),
    ));
}

#[cfg(feature = "server")]
fn init_app(app: &mut App) {
    println!("initializing server ...");

    app.add_plugins((
        MinimalPlugins.set(bevy::app::ScheduleRunnerPlugin::run_loop(
            bevy::utils::Duration::from_secs_f64(SERVER_TICK_RATE),
        )),
    ));
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        Name::new("Main Camera"),
    ));

    // ground
    commands.spawn((
        Collider::cuboid(100.0, 0.1, 100.0),
        TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)),
        Name::new("Ground"),
    ));

    // bouncing ball
    commands.spawn((
        RigidBody::Dynamic,
        Collider::ball(0.5),
        Restitution::coefficient(0.7),
        TransformBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)),
        Name::new("Ball"),
    ));
}

fn main() {
    let mut app = App::new();
    init_app(&mut app);

    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_systems(Startup, setup);

    info!("running ...");
    app.run();
}
