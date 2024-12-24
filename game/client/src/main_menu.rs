use bevy::prelude::*;
use bevy_replicon::prelude::*;

use game_common::{
    cleanup_state,
    network::{ConnectEvent, PlayerClientId},
};

use crate::{client, options::Options, ui, AppState};

#[derive(Debug, Component)]
struct OnMainMenu;

#[derive(Debug)]
pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), enter)
            .add_systems(
                OnExit(AppState::MainMenu),
                (exit, cleanup_state::<OnMainMenu>, cleanup_state::<Node>),
            );
    }
}

fn on_start_local(
    event: Trigger<Pointer<Click>>,
    mut commands: Commands,
    options: Res<Options>,
    client: Res<client::ClientState>,
    mut evw_connect: EventWriter<ConnectEvent>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    if !ui::check_click_event(
        event.entity(),
        event.target,
        event.button,
        PointerButton::Primary,
    ) {
        return;
    }

    let client_id = PlayerClientId::new(ClientId::SERVER);
    commands.insert_resource(client_id);
    client::on_connected_server(
        &client,
        client_id,
        options.user_id,
        &mut evw_connect,
        &mut app_state,
    );
}

fn on_find_server(event: Trigger<Pointer<Click>>, mut app_state: ResMut<NextState<AppState>>) {
    if !ui::check_click_event(
        event.entity(),
        event.target,
        event.button,
        PointerButton::Primary,
    ) {
        return;
    }

    app_state.set(AppState::ConnectToServer);
}

fn on_exit_game(event: Trigger<Pointer<Click>>, mut exit: EventWriter<AppExit>) {
    if !ui::check_click_event(
        event.entity(),
        event.target,
        event.button,
        PointerButton::Primary,
    ) {
        return;
    }

    exit.send(AppExit::Success);
}

fn enter(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("entering main menu ...");

    commands.insert_resource(ClearColor(Color::BLACK));

    commands.spawn((Camera2d, IsDefaultUiCamera, OnMainMenu));

    ui::spawn_canvas(&mut commands, "Main Menu").with_children(|parent| {
        ui::spawn_button(parent, &asset_server, "Start Local").observe(on_start_local);
        ui::spawn_button(parent, &asset_server, "Find Server").observe(on_find_server);
        ui::spawn_button(parent, &asset_server, "Exit").observe(on_exit_game);
    });
}

fn exit(mut commands: Commands) {
    info!("exiting main menu ...");

    commands.remove_resource::<ClearColor>();
}
