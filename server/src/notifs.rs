use bevy::prelude::*;
use bevy_tokio_tasks::TokioTasksRuntime;
use futures_lite::future;
use futures_util::StreamExt;
use http::uri::Uri;
use tokio::{net::TcpStream, task};
use tokio_tungstenite::{tungstenite::handshake::client::Request, MaybeTlsStream, WebSocketStream};

#[derive(Debug, Component)]
pub struct SubscribeNotifs(pub Option<Request>);

#[derive(Debug, Component)]
struct SubscribeNotifsTask(pub (Uri, task::JoinHandle<Result<(), anyhow::Error>>));

// TODO: unsubscribe

#[derive(Debug, Component)]
struct ListenNotifs(pub (Uri, Option<WebSocketStream<MaybeTlsStream<TcpStream>>>));

#[derive(Debug, Component)]
struct ListenNotifsTask(pub (Uri, task::JoinHandle<Result<(), anyhow::Error>>));

#[derive(Event)]
pub struct NotifsSubscribeResult(pub (Uri, bool));

#[derive(Event)]
pub struct NotifsDisconnected(pub Uri);

#[derive(Event)]
pub struct Notification(pub (Uri, tokio_tungstenite::tungstenite::protocol::Message));

pub struct NotifsPlugin;

impl Plugin for NotifsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NotifsSubscribeResult>()
            .add_event::<NotifsDisconnected>()
            .add_event::<Notification>()
            .add_systems(
                Update,
                (
                    (subscribe_notifs, poll_subscribe_notifs),
                    (listen_notifs, poll_listen_notifs),
                ),
            );
    }
}

fn subscribe_notifs(
    mut commands: Commands,
    mut requests: Query<(Entity, &mut SubscribeNotifs), Added<SubscribeNotifs>>,
    runtime: Res<TokioTasksRuntime>,
) {
    for (entity, mut request) in requests.iter_mut() {
        if request.0.is_none() {
            continue;
        }

        let request = request.0.take().unwrap();
        let uri = request.uri().clone();
        let task = runtime.spawn_background_task(|mut ctx| async move {
            let uri = request.uri().clone();
            info!("subscribing to notifications from {}", uri);

            // TODO: error handle this (if it fails, send a NotifsSubscribeResult event)
            let (stream, _) = tokio_tungstenite::connect_async(request).await?;

            ctx.run_on_main_thread(move |ctx| {
                ctx.world.spawn(ListenNotifs((uri, Some(stream))));
            })
            .await;

            Ok(())
        });

        commands
            .entity(entity)
            .insert(SubscribeNotifsTask((uri, task)))
            .remove::<SubscribeNotifs>();
    }
}

fn poll_subscribe_notifs(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SubscribeNotifsTask)>,
    mut notifs_subscribed_events: EventWriter<NotifsSubscribeResult>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(response) = future::block_on(future::poll_once(&mut task.0 .1)) {
            let uri = &task.0 .0;

            // TODO: error handling
            let response = response.unwrap();

            match response {
                Ok(_) => {
                    debug!("subscribed to notifications from {}", uri);
                    notifs_subscribed_events.send(NotifsSubscribeResult((uri.clone(), true)));
                }
                Err(e) => {
                    warn!("failed to subscribe to notifications from {}: {}", uri, e);
                    notifs_subscribed_events.send(NotifsSubscribeResult((uri.clone(), false)));
                }
            }

            commands.entity(entity).despawn();
        }
    }
}

fn listen_notifs(
    mut commands: Commands,
    mut requests: Query<(Entity, &mut ListenNotifs), Added<ListenNotifs>>,
    runtime: Res<TokioTasksRuntime>,
) {
    for (entity, mut request) in requests.iter_mut() {
        let uri = request.0 .0.clone();
        let stream = request.0 .1.take().unwrap();
        let task = runtime.spawn_background_task(|mut ctx| async move {
            let (_, mut read) = stream.split();
            while let Some(Ok(msg)) = read.next().await {
                let uri = uri.clone();
                debug!("got notification from {}: {}", uri, msg);
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world.send_event(Notification((uri, msg)));
                })
                .await;
            }

            warn!("{} notifications connection closed", uri);
            ctx.run_on_main_thread(move |ctx| {
                ctx.world.send_event(NotifsDisconnected(uri));
            })
            .await;

            Ok(())
        });

        commands
            .entity(entity)
            .insert(ListenNotifsTask((request.0 .0.clone(), task)))
            .remove::<SubscribeNotifs>();
    }
}

fn poll_listen_notifs(mut commands: Commands, mut tasks: Query<(Entity, &mut ListenNotifsTask)>) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(response) = future::block_on(future::poll_once(&mut task.0 .1)) {
            // TODO: error handling
            let response = response.unwrap();

            // TODO: error handling
            response.unwrap();

            debug!(
                "finished listening for notifications from from {}",
                task.0 .0
            );

            commands.entity(entity).despawn();
        }
    }
}
