use avian3d::prelude::*;
use bevy::{color::palettes::css, prelude::*};
use bevy_replicon::prelude::*;

use crate::{
    cleanup_state, dynamic,
    network::{ConnectEvent, InputUpdateEvent, PlayerJumpEvent},
    player, spawn, world, GameAssetState, GameState, InputState,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct ServerSet;

#[derive(Debug, Default, Component)]
pub struct OnInGame;

#[derive(Debug)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // third-party plugins
            PhysicsPlugins::default(), // TODO: this doesn't work with tnua: .set(PhysicsInterpolationPlugin::interpolate_all()),
            bevy_tnua::controller::TnuaControllerPlugin::new(PhysicsSchedule),
            bevy_tnua_avian3d::TnuaAvian3dPlugin::new(PhysicsSchedule),
            // game plugins
            player::PlayerPlugin,
            dynamic::DynamicPlugin,
        ))
        .init_state::<GameState>()
        .init_resource::<InputState>()
        .add_systems(OnEnter(GameState::LoadAssets), load_assets)
        .add_systems(
            Update,
            wait_for_assets.run_if(in_state(GameState::LoadAssets)),
        )
        .add_systems(OnEnter(GameState::SpawnWorld), spawn_world)
        .add_systems(
            Update,
            wait_for_world.run_if(in_state(GameState::SpawnWorld)),
        )
        .add_systems(
            OnEnter(GameState::InGame),
            (
                enter_server
                    .run_if(server_or_singleplayer)
                    .in_set(ServerSet),
                enter_client.run_if(client_connected),
            ),
        )
        .add_systems(
            PreUpdate,
            (handle_input_update, handle_jump_event)
                .after(bevy_replicon::server::ServerSet::Receive)
                .run_if(in_state(GameState::InGame))
                .run_if(server_or_singleplayer),
        )
        .add_systems(
            OnExit(GameState::InGame),
            (exit, cleanup_state::<OnInGame>, cleanup_state::<Node>),
        );

        app.register_type::<InputState>();

        // TOOD: move to a network plugin
        app.add_client_event::<ConnectEvent>(ChannelKind::Unordered)
            .add_client_event::<InputUpdateEvent>(ChannelKind::Ordered)
            .add_client_event::<PlayerJumpEvent>(ChannelKind::Unordered);
    }
}

fn load_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    info!("loading assets ...");

    let mut game_assets = GameAssetState::default();

    world::load_assets(&mut meshes, &mut materials, &mut game_assets);
    dynamic::load_assets(&mut meshes, &mut materials, &mut game_assets);
    player::load_assets(
        &mut meshes,
        &mut materials,
        &mut animations,
        &mut graphs,
        &mut game_assets,
    );

    commands.insert_resource(game_assets);
}

fn wait_for_assets(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::SpawnWorld);
}

// TODO: it would be nice if we could not load materials on the server

fn spawn_world(mut commands: Commands, assets: Res<GameAssetState>) {
    info!("spawning world ...");

    commands.insert_resource(AmbientLight {
        color: css::WHITE.into(),
        brightness: 80.0,
    });

    world::spawn_directional_light(
        &mut commands,
        css::ORANGE_RED.into(),
        Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            rotation: Quat::from_rotation_x(-45.0f32.to_radians()),
            ..default()
        },
        "Sun",
    );

    // floor
    world::spawn_wall(
        &mut commands,
        Transform::from_xyz(0.0, 0.0, 0.0),
        assets.floor_mesh.clone(),
        assets.floor_material.clone(),
        "Ground",
    );

    // ceiling
    world::spawn_wall(
        &mut commands,
        Transform {
            translation: Vec3::new(0.0, 50.0, 0.0),
            rotation: Quat::from_rotation_z(180.0f32.to_radians()),
            ..default()
        },
        assets.floor_mesh.clone(),
        assets.floor_material.clone(),
        "Ceiling",
    );

    // left wall
    world::spawn_wall(
        &mut commands,
        Transform {
            translation: Vec3::new(-25.0, 25.0, 0.0),
            rotation: Quat::from_rotation_z(-90.0f32.to_radians()),
            ..default()
        },
        assets.floor_mesh.clone(),
        assets.wall_material.clone(),
        "Left Wall",
    );

    // right wall
    world::spawn_wall(
        &mut commands,
        Transform {
            translation: Vec3::new(25.0, 25.0, 0.0),
            rotation: Quat::from_rotation_z(90.0f32.to_radians()),
            ..default()
        },
        assets.floor_mesh.clone(),
        assets.wall_material.clone(),
        "Right Wall",
    );

    // forward wall
    world::spawn_wall(
        &mut commands,
        Transform {
            translation: Vec3::new(0.0, 25.0, -25.0),
            rotation: Quat::from_rotation_x(90.0f32.to_radians()),
            ..default()
        },
        assets.floor_mesh.clone(),
        assets.wall_material.clone(),
        "Forward Wall",
    );

    // rear wall
    world::spawn_wall(
        &mut commands,
        Transform {
            translation: Vec3::new(0.0, 25.0, 25.0),
            rotation: Quat::from_rotation_x(-90.0f32.to_radians()),
            ..default()
        },
        assets.floor_mesh.clone(),
        assets.wall_material.clone(),
        "Rear Wall",
    );

    // player spawns
    commands.spawn((
        spawn::SpawnPoint,
        Transform::from_translation(Vec3::new(-5.0, 2.1, 5.0)),
    ));
}

fn wait_for_world(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::InGame);
}

fn enter_server(mut commands: Commands, assets: Res<GameAssetState>) {
    info!("entering game (server / singleplayer) ...");

    dynamic::spawn_ball(&mut commands, Vec3::new(0.0, 20.0, -5.0), &assets);
}

pub fn spawn_client_world(commands: &mut Commands) {
    info!("spawning client world ...");

    commands.insert_resource(ClearColor(Color::BLACK));
}

fn enter_client(mut commands: Commands) {
    info!("entering game (client) ...");

    spawn_client_world(&mut commands);
}

fn exit(mut commands: Commands) {
    info!("exiting game ...");

    commands.remove_resource::<ClearColor>();
    commands.remove_resource::<AmbientLight>();
    commands.remove_resource::<GameAssetState>();
}

fn handle_input_update(
    mut evr_input_update: EventReader<FromClient<InputUpdateEvent>>,
    mut player_query: Query<(&mut player::LastInput, &player::Player)>,
) {
    for FromClient { client_id, event } in evr_input_update.read() {
        // validation handled by server

        for (mut last_input, player) in &mut player_query {
            if player.client_id == *client_id {
                last_input.input_state = event.0;
            }
        }
    }
}

fn handle_jump_event(
    mut evr_jump: EventReader<FromClient<PlayerJumpEvent>>,
    mut player_query: Query<(&mut player::LastInput, &player::Player)>,
) {
    for FromClient {
        client_id,
        event: _,
    } in evr_jump.read()
    {
        // validation handled by server

        for (mut last_input, player) in &mut player_query {
            if player.client_id == *client_id {
                last_input.jump = true;
            }
        }
    }
}
