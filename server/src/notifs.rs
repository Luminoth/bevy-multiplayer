use bevy::{
    ecs::system::{EntityCommands, IntoObserverSystem, SystemParam},
    prelude::*,
};
use bevy_tokio_tasks::TokioTasksRuntime;
use futures_lite::future;
use futures_util::StreamExt;
use http::uri::Uri;
use tokio::{net::TcpStream, task};
use tokio_tungstenite::{tungstenite::handshake::client::Request, MaybeTlsStream, WebSocketStream};

#[derive(Debug, Component)]
struct SubscribeNotifs(pub Option<Request>);

#[derive(Debug, Component)]
struct SubscribeNotifsTask(pub (Uri, task::JoinHandle<Result<(), anyhow::Error>>));

// TODO: unsubscribe

#[derive(Debug, Component)]
struct ListenNotifs(pub (Uri, Option<WebSocketStream<MaybeTlsStream<TcpStream>>>));

#[derive(Debug, Component)]
struct ListenNotifsTask(pub (Uri, task::JoinHandle<Result<(), anyhow::Error>>));

#[allow(dead_code)]
#[derive(Debug, Event)]
pub struct NotifsSubscribeSuccess {
    pub uri: Uri,
}

#[allow(dead_code)]
#[derive(Debug, Event)]
pub struct NotifsError {
    pub uri: Uri,
    pub error: anyhow::Error,
}

#[allow(dead_code)]
#[derive(Debug, Event)]
pub struct NotifsDisconnected {
    pub uri: Uri,
}

#[allow(dead_code)]
#[derive(Debug, Event)]
pub struct Notification {
    pub uri: Uri,
    pub message: tokio_tungstenite::tungstenite::protocol::Message,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct NotifsSet;

pub struct NotifsPlugin;

impl Plugin for NotifsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                (subscribe_notifs, poll_subscribe_notifs)
                    .chain()
                    .in_set(NotifsSet),
                (listen_notifs, poll_listen_notifs)
                    .chain()
                    .in_set(NotifsSet),
            ),
        );
    }
}

pub struct NotifSubscriptionBuilder<'a>(EntityCommands<'a>);

impl<'a> NotifSubscriptionBuilder<'a> {
    pub fn on_success<RB: Bundle, RM, OR: IntoObserverSystem<NotifsSubscribeSuccess, RB, RM>>(
        &mut self,
        onsuccess: OR,
    ) -> &mut Self {
        self.0.observe(onsuccess);
        self
    }

    pub fn on_error<RB: Bundle, RM, OR: IntoObserverSystem<NotifsError, RB, RM>>(
        &mut self,
        onerror: OR,
    ) -> &mut Self {
        self.0.observe(onerror);
        self
    }

    pub fn on_notif<RB: Bundle, RM, OR: IntoObserverSystem<Notification, RB, RM>>(
        &mut self,
        onnotif: OR,
    ) -> &mut Self {
        self.0.observe(onnotif);
        self
    }

    pub fn on_disconnect<RB: Bundle, RM, OR: IntoObserverSystem<NotifsDisconnected, RB, RM>>(
        &mut self,
        ondisconnect: OR,
    ) -> &mut Self {
        self.0.observe(ondisconnect);
        self
    }
}

#[derive(SystemParam)]
pub struct NotifSubscriber<'w, 's> {
    commands: Commands<'w, 's>,
}

impl<'w, 's> NotifSubscriber<'w, 's> {
    pub fn subscribe(&mut self, req: Request) -> NotifSubscriptionBuilder {
        let inflight = SubscribeNotifs(Some(req));
        NotifSubscriptionBuilder(self.commands.spawn(inflight))
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

        let entity = entity.clone();
        let request = request.0.take().unwrap();
        let uri = request.uri().clone();
        let task = runtime.spawn_background_task(move |mut ctx| async move {
            let uri = request.uri().clone();
            info!("subscribing to notifications from {}", uri);

            // TODO: error handle this (if it fails, trigger a NotifsError event)
            let (stream, _) = tokio_tungstenite::connect_async(request).await?;

            // start listening to the stream
            ctx.run_on_main_thread(move |ctx| {
                ctx.world
                    .entity_mut(entity)
                    .insert(ListenNotifs((uri, Some(stream))));
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
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(response) = future::block_on(future::poll_once(&mut task.0 .1)) {
            let uri = &task.0 .0;

            // TODO: error handling
            let response = response.unwrap();

            match response {
                Ok(_) => {
                    debug!("subscribed to notifications from {}", uri);
                    commands.trigger_targets(
                        NotifsSubscribeSuccess { uri: uri.clone() },
                        entity.clone(),
                    );
                }
                Err(err) => {
                    warn!("failed to subscribe to notifications from {}: {}", uri, err);
                    commands.trigger_targets(
                        NotifsError {
                            uri: uri.clone(),
                            error: err,
                        },
                        entity.clone(),
                    );
                }
            }

            commands.entity(entity).remove::<SubscribeNotifsTask>();
        }
    }
}

fn listen_notifs(
    mut commands: Commands,
    mut requests: Query<(Entity, &mut ListenNotifs), Added<ListenNotifs>>,
    runtime: Res<TokioTasksRuntime>,
) {
    for (entity, mut request) in requests.iter_mut() {
        let entity = entity.clone();
        let uri = request.0 .0.clone();
        let stream = request.0 .1.take().unwrap();
        let task = runtime.spawn_background_task(move |mut ctx| async move {
            let (_, mut read) = stream.split();
            while let Some(Ok(msg)) = read.next().await {
                let uri = uri.clone();
                debug!("got notification from {}: {}", uri, msg);
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world
                        .trigger_targets(Notification { uri, message: msg }, entity.clone());
                })
                .await;
            }

            warn!("{} notifications connection closed", uri);
            ctx.run_on_main_thread(move |ctx| {
                ctx.world
                    .trigger_targets(NotifsDisconnected { uri }, entity.clone());
            })
            .await;

            Ok(())
        });

        commands
            .entity(entity)
            .insert(ListenNotifsTask((request.0 .0.clone(), task)))
            .remove::<ListenNotifs>();
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
