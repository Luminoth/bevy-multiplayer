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
struct ConnectWebSocket(pub Option<Request>);

#[derive(Debug, Component)]
struct ConnectWebSocketTask(pub (Uri, task::JoinHandle<Result<(), anyhow::Error>>));

#[derive(Debug, Component)]
struct ListenWebSocket(pub (Uri, Option<WebSocketStream<MaybeTlsStream<TcpStream>>>));

#[derive(Debug, Component)]
struct ListenWebSocketTask(pub (Uri, task::JoinHandle<Result<(), anyhow::Error>>));

// TODO: disconnect

#[allow(dead_code)]
#[derive(Debug, Event)]
pub struct WebSocketConnectSuccessEvent {
    pub uri: Uri,
}

#[allow(dead_code)]
#[derive(Debug, Event)]
pub struct WebSocketErrorEvent {
    pub uri: Uri,
    pub error: anyhow::Error,
}

#[allow(dead_code)]
#[derive(Debug, Event)]
pub struct WebSocketDisconnectEvent {
    pub uri: Uri,
}

#[allow(dead_code)]
#[derive(Debug, Event)]
pub struct WebSocketMessageEvent {
    pub uri: Uri,
    pub message: tokio_tungstenite::tungstenite::protocol::Message,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct WebSocketSet;

pub struct WebSocketPlugin;

impl Plugin for WebSocketPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                (connect_websockets, poll_connect_websockets)
                    .chain()
                    .in_set(WebSocketSet),
                (listen_websockets, poll_listen_websockets)
                    .chain()
                    .in_set(WebSocketSet),
            ),
        );
    }
}

pub struct WebSocketBuilder<'a>(EntityCommands<'a>);

impl<'a> WebSocketBuilder<'a> {
    pub fn on_success<
        RB: Bundle,
        RM,
        OR: IntoObserverSystem<WebSocketConnectSuccessEvent, RB, RM>,
    >(
        mut self,
        onconnect: OR,
    ) -> Self {
        self.0.observe(onconnect);
        self
    }

    pub fn on_error<RB: Bundle, RM, OR: IntoObserverSystem<WebSocketErrorEvent, RB, RM>>(
        mut self,
        onerror: OR,
    ) -> Self {
        self.0.observe(onerror);
        self
    }

    pub fn on_message<RB: Bundle, RM, OR: IntoObserverSystem<WebSocketMessageEvent, RB, RM>>(
        mut self,
        onmessage: OR,
    ) -> Self {
        self.0.observe(onmessage);
        self
    }

    pub fn on_disconnect<
        RB: Bundle,
        RM,
        OR: IntoObserverSystem<WebSocketDisconnectEvent, RB, RM>,
    >(
        mut self,
        ondisconnect: OR,
    ) -> Self {
        self.0.observe(ondisconnect);
        self
    }
}

#[derive(SystemParam)]
pub struct WebSocketClient<'w, 's> {
    commands: Commands<'w, 's>,
}

impl<'w, 's> WebSocketClient<'w, 's> {
    pub fn connect(&mut self, req: Request) -> WebSocketBuilder {
        let inflight = ConnectWebSocket(Some(req));
        WebSocketBuilder(self.commands.spawn(inflight))
    }
}

fn connect_websockets(
    mut commands: Commands,
    mut requests: Query<(Entity, &mut ConnectWebSocket), Added<ConnectWebSocket>>,
    runtime: Res<TokioTasksRuntime>,
) {
    for (entity, mut request) in requests.iter_mut() {
        if request.0.is_none() {
            continue;
        }

        let request = request.0.take().unwrap();
        let uri = request.uri().clone();
        let task = runtime.spawn_background_task(move |mut ctx| async move {
            let uri = request.uri().clone();
            info!("connecting websocket at {}", uri);

            // TODO: error handle this (if it fails, trigger a WebSocketErrorEvent)
            let (stream, _) = tokio_tungstenite::connect_async(request).await?;

            // start listening to the stream
            ctx.run_on_main_thread(move |ctx| {
                ctx.world
                    .entity_mut(entity)
                    .insert(ListenWebSocket((uri, Some(stream))));
            })
            .await;

            Ok(())
        });

        commands
            .entity(entity)
            .insert(ConnectWebSocketTask((uri, task)))
            .remove::<ConnectWebSocket>();
    }
}

fn poll_connect_websockets(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ConnectWebSocketTask)>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(response) = future::block_on(future::poll_once(&mut task.0 .1)) {
            let uri = &task.0 .0;

            // TODO: error handling
            let response = response.unwrap();

            match response {
                Ok(_) => {
                    debug!("connected websocket at {}", uri);
                    commands
                        .trigger_targets(WebSocketConnectSuccessEvent { uri: uri.clone() }, entity);
                }
                Err(err) => {
                    warn!("failed to connect websocket at {}: {}", uri, err);
                    commands.trigger_targets(
                        WebSocketErrorEvent {
                            uri: uri.clone(),
                            error: err,
                        },
                        entity,
                    );
                }
            }

            commands.entity(entity).remove::<ConnectWebSocketTask>();
        }
    }
}

fn listen_websockets(
    mut commands: Commands,
    mut requests: Query<(Entity, &mut ListenWebSocket), Added<ListenWebSocket>>,
    runtime: Res<TokioTasksRuntime>,
) {
    for (entity, mut request) in requests.iter_mut() {
        let uri = request.0 .0.clone();
        let stream = request.0 .1.take().unwrap();
        let task = runtime.spawn_background_task(move |mut ctx| async move {
            let (_, mut read) = stream.split();
            while let Some(Ok(msg)) = read.next().await {
                let uri = uri.clone();
                debug!("got websocket message from {}: {}", uri, msg);
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world
                        .trigger_targets(WebSocketMessageEvent { uri, message: msg }, entity);
                })
                .await;
            }

            warn!("websocket connection {} closed!", uri);
            ctx.run_on_main_thread(move |ctx| {
                ctx.world
                    .trigger_targets(WebSocketDisconnectEvent { uri }, entity);
            })
            .await;

            Ok(())
        });

        commands
            .entity(entity)
            .insert(ListenWebSocketTask((request.0 .0.clone(), task)))
            .remove::<ListenWebSocket>();
    }
}

fn poll_listen_websockets(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ListenWebSocketTask)>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(response) = future::block_on(future::poll_once(&mut task.0 .1)) {
            // TODO: error handling
            let response = response.unwrap();

            // TODO: error handling
            response.unwrap();

            info!("closing websocket connection to {}", task.0 .0);

            commands.entity(entity).despawn();
        }
    }
}
