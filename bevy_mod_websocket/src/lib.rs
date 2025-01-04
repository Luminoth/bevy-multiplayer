//! Library for handling WebSocket connections with [`bevy`](https://bevyengine.org/)
//!
//! This library requires the [`bevy_tokio_tasks::TokioTasksPlugin`] be registered with the [`bevy::app::App`]

#![deny(missing_docs)]
#![deny(warnings)]

use bevy::{
    ecs::system::{EntityCommands, IntoObserverSystem, SystemParam},
    prelude::*,
    utils::Duration,
};
use bevy_tokio_tasks::TokioTasksRuntime;
use futures_lite::future;
use futures_util::StreamExt;
use http::uri::Uri;
use thiserror::Error;
use tokio::{net::TcpStream, task};
use tokio_tungstenite::{tungstenite::handshake::client::Request, MaybeTlsStream, WebSocketStream};

// TODO: support intential disconnect
// when disconnecting, we need to make sure we despawn the entity

// TODO: a lot of cloning from internal Components could be moved
// to use the same Option::take() pattern used with ConnectWebSocket

/// Error returned by WebSocket connections
#[derive(Error, Debug)]
pub enum WebSocketError {
    /// An error occurred with the WebSocket connection
    #[error("websocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    /// An error occurred in the [`tokio`] runtime
    #[error("task error")]
    TaskError,

    /// An unknown error occurred
    #[error("unknown websocket error")]
    Unknown,
}

#[derive(Debug, Component)]
struct ConnectWebSocket {
    request: Option<Request>,
    timer: Option<Timer>,
}

impl ConnectWebSocket {
    #[inline]
    fn new(request: Request) -> Self {
        Self {
            request: Some(request),
            timer: None,
        }
    }

    #[inline]
    fn with_timer(request: Request, duration: Duration) -> Self {
        Self {
            request: Some(request),
            timer: Some(Timer::new(duration, TimerMode::Once)),
        }
    }

    #[inline]
    fn tick(&mut self, time: &Time<Real>) {
        self.timer.as_mut().map(|timer| timer.tick(time.delta()));
    }

    #[inline]
    fn is_ready(&self) -> bool {
        self.timer
            .as_ref()
            .map(|timer| timer.finished())
            .unwrap_or(true)
    }
}

#[derive(Debug, Component)]
struct ConnectWebSocketTask {
    request: Request,
    task: task::JoinHandle<Result<(), WebSocketError>>,
}

#[derive(Debug, Component)]
struct ListenWebSocket {
    request: Request,
    stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl ListenWebSocket {
    #[inline]
    fn new(request: Request, stream: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self {
            request,
            stream: Some(stream),
        }
    }
}

#[derive(Debug, Component)]
struct ListenWebSocketTask {
    request: Request,
    task: task::JoinHandle<Result<(), WebSocketError>>,
}

/// WebSocket connection success event
#[derive(Debug, Event)]
pub struct WebSocketConnectSuccessEvent {
    /// The connection [`Uri`]
    pub uri: Uri,
}

/// WebSocket error event
#[derive(Debug, Event)]
pub struct WebSocketErrorEvent {
    /// The connection [`Request`]
    pub request: Request,

    /// The [WebSocketError] that occurred
    pub error: WebSocketError,
}

/// WebSocket disconnect event
#[derive(Debug, Event)]
pub struct WebSocketDisconnectEvent {
    /// The connection [`Request`]
    pub request: Request,
}

/// A WebSocket message
#[derive(Debug, Clone)]
pub enum Message {
    /// Text message
    Text(String),

    /// Binary message
    Binary(Vec<u8>),

    /// WebSocket Ping message
    Ping(Vec<u8>),

    /// WebSocket Pong message
    Pong(Vec<u8>),

    /// WebSocket close message
    Close,
}

impl From<tokio_tungstenite::tungstenite::protocol::Message> for Message {
    fn from(message: tokio_tungstenite::tungstenite::protocol::Message) -> Self {
        match message {
            tokio_tungstenite::tungstenite::protocol::Message::Text(text) => {
                Message::Text(text.to_string())
            }
            tokio_tungstenite::tungstenite::protocol::Message::Binary(bin) => {
                Message::Binary(bin.to_vec())
            }
            tokio_tungstenite::tungstenite::protocol::Message::Ping(ping) => {
                Message::Ping(ping.to_vec())
            }
            tokio_tungstenite::tungstenite::protocol::Message::Pong(pong) => {
                Message::Pong(pong.to_vec())
            }
            tokio_tungstenite::tungstenite::protocol::Message::Close(_) => Message::Close,
            tokio_tungstenite::tungstenite::protocol::Message::Frame(_) => unreachable!(),
        }
    }
}

/// WebSocket message event
#[derive(Debug, Event)]
pub struct WebSocketMessageEvent {
    /// The connection [`Uri`]
    pub uri: Uri,

    /// The message that was received
    pub message: Message,
}

/// The WebSocket [`SystemSet`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct WebSocketSet;

/// Add this plugin, along with the [`bevy_tokio_tasks::TokioTasksPlugin`] to enable WebSocket connections
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

/// Builder to connect observers
pub struct WebSocketBuilder<'a>(EntityCommands<'a>);

impl WebSocketBuilder<'_> {
    /// Triggered on WebSocket connection success
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

    /// Triggered on WebSocket connection error
    pub fn on_error<RB: Bundle, RM, OR: IntoObserverSystem<WebSocketErrorEvent, RB, RM>>(
        mut self,
        onerror: OR,
    ) -> Self {
        self.0.observe(onerror);
        self
    }

    /// Triggered on WebSocket message
    pub fn on_message<RB: Bundle, RM, OR: IntoObserverSystem<WebSocketMessageEvent, RB, RM>>(
        mut self,
        onmessage: OR,
    ) -> Self {
        self.0.observe(onmessage);
        self
    }

    /// Triggered on WebSocket disconnect
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

/// The WebSocket client [`SystemParam`]
#[derive(SystemParam)]
pub struct WebSocketClient<'w, 's> {
    commands: Commands<'w, 's>,
}

impl WebSocketClient<'_, '_> {
    /// Starts an attempt at a WebSocket connection
    pub fn connect(&mut self, req: Request) -> WebSocketBuilder {
        debug!("starting websocket connect");

        let inflight = ConnectWebSocket::new(req);

        let mut commands = self.commands.spawn(inflight);
        commands.insert(Name::new("websocket"));
        WebSocketBuilder(commands)
    }

    /// Starts an attempt at a WebSocket connection after a timer completes
    pub fn retry(&mut self, entity: Entity, req: Request, duration: Duration) -> WebSocketBuilder {
        debug!("retrying websocket connect in {:?}", duration);

        let inflight = ConnectWebSocket::with_timer(req, duration);

        let mut commands = self.commands.entity(entity);
        commands.insert(inflight);
        WebSocketBuilder(commands)
    }
}

fn connect_websockets(
    mut commands: Commands,
    mut requests: Query<(Entity, &mut ConnectWebSocket)>,
    rt: Res<Time<Real>>,
    runtime: Res<TokioTasksRuntime>,
) {
    for (entity, mut request) in requests.iter_mut() {
        if request.request.is_none() {
            warn!("connect websocket missing request!");
            commands.entity(entity).remove::<ConnectWebSocket>();
            continue;
        }

        request.tick(&rt);
        if !request.is_ready() {
            continue;
        }

        let request = request.request.take().unwrap();
        let task = runtime.spawn_background_task({
            let request = request.clone();
            move |mut ctx| async move {
                info!("connecting websocket at {} ...", request.uri());

                match tokio_tungstenite::connect_async(request.clone()).await {
                    Ok((stream, _)) => {
                        // start listening to the stream
                        ctx.run_on_main_thread(move |ctx| {
                            ctx.world
                                .entity_mut(entity)
                                .insert(ListenWebSocket::new(request, stream));
                        })
                        .await;

                        Ok(())
                    }
                    Err(err) => Err(WebSocketError::from(err)),
                }
            }
        });

        commands
            .entity(entity)
            .insert(ConnectWebSocketTask { request, task })
            .remove::<ConnectWebSocket>();
    }
}

fn poll_connect_websockets(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ConnectWebSocketTask)>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(response) = future::block_on(future::poll_once(&mut task.task)) {
            let uri = task.request.uri();

            match response {
                Ok(response) => match response {
                    Ok(_) => {
                        debug!("connected websocket at {}", uri);
                        commands.trigger_targets(
                            WebSocketConnectSuccessEvent { uri: uri.clone() },
                            entity,
                        );
                    }
                    Err(err) => {
                        warn!("failed to connect websocket at {}: {}", uri, err);
                        commands.trigger_targets(
                            WebSocketErrorEvent {
                                request: task.request.clone(),
                                error: err,
                            },
                            entity,
                        );
                    }
                },
                Err(err) => {
                    error!("failed to join connect websocket task at {}: {}", uri, err);
                    commands.trigger_targets(
                        WebSocketErrorEvent {
                            request: task.request.clone(),
                            error: WebSocketError::TaskError,
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
    mut websockets: Query<(Entity, &mut ListenWebSocket)>,
    runtime: Res<TokioTasksRuntime>,
) {
    for (entity, mut websocket) in websockets.iter_mut() {
        if websocket.stream.is_none() {
            warn!("listen websocket missing stream!");
            commands.entity(entity).remove::<ListenWebSocket>();
            continue;
        }

        let request = websocket.request.clone();
        let stream = websocket.stream.take().unwrap();
        let task = runtime.spawn_background_task({
            let request = request.clone();
            move |mut ctx| async move {
                info!("listening websocket at {} ...", request.uri());

                let (_, mut read) = stream.split();
                while let Some(Ok(msg)) = read.next().await {
                    debug!("got websocket message from {}: {}", request.uri(), msg);
                    ctx.run_on_main_thread({
                        let uri = request.uri().clone();
                        move |ctx| {
                            ctx.world.trigger_targets(
                                WebSocketMessageEvent {
                                    uri,
                                    message: msg.into(),
                                },
                                entity,
                            );
                        }
                    })
                    .await;
                }

                warn!("websocket connection {} closed!", request.uri());
                ctx.run_on_main_thread(move |ctx| {
                    ctx.world
                        .trigger_targets(WebSocketDisconnectEvent { request }, entity);
                })
                .await;

                Ok(())
            }
        });

        commands
            .entity(entity)
            .insert(ListenWebSocketTask { request, task })
            .remove::<ListenWebSocket>();
    }
}

fn poll_listen_websockets(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ListenWebSocketTask)>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(response) = future::block_on(future::poll_once(&mut task.task)) {
            match response {
                Ok(response) => {
                    if let Err(err) = response {
                        warn!(
                            "error listening websocket at {}: {}",
                            task.request.uri(),
                            err
                        );
                        commands.trigger_targets(
                            WebSocketErrorEvent {
                                request: task.request.clone(),
                                error: err,
                            },
                            entity,
                        );

                        continue;
                    }
                }
                Err(err) => {
                    error!(
                        "failed to join listen websocket task at {}: {}",
                        task.request.uri(),
                        err
                    );
                    commands.trigger_targets(
                        WebSocketErrorEvent {
                            request: task.request.clone(),
                            error: WebSocketError::TaskError,
                        },
                        entity,
                    );
                }
            }

            commands.entity(entity).remove::<ListenWebSocketTask>();
        }
    }
}
