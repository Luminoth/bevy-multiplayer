use bevy::{
    ecs::system::{EntityCommands, IntoObserverSystem, SystemParam},
    prelude::*,
    utils::Duration,
};
use bevy_tokio_tasks::TokioTasksRuntime;
use futures_lite::future;
use futures_util::StreamExt;
use http::uri::Uri;
use tokio::{net::TcpStream, task};
use tokio_tungstenite::{tungstenite::handshake::client::Request, MaybeTlsStream, WebSocketStream};

#[derive(Debug, Component)]
struct ConnectWebSocket {
    request: Option<Request>,

    // TODO: handling backoff (reconnects, etc) needs to be finished up
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

    #[allow(dead_code)]
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
    uri: Uri,
    task: task::JoinHandle<Result<(), anyhow::Error>>,
}

#[derive(Debug, Component)]
struct ListenWebSocket {
    uri: Uri,
    stream: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl ListenWebSocket {
    #[inline]
    fn new(uri: Uri, stream: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self {
            uri,
            stream: Some(stream),
        }
    }
}

#[derive(Debug, Component)]
struct ListenWebSocketTask {
    uri: Uri,
    task: task::JoinHandle<Result<(), anyhow::Error>>,
}

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
        let inflight = ConnectWebSocket::new(req);
        WebSocketBuilder(self.commands.spawn(inflight))
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
                    .insert(ListenWebSocket::new(uri, stream));
            })
            .await;

            Ok(())
        });

        commands
            .entity(entity)
            .insert(ConnectWebSocketTask { uri, task })
            .remove::<ConnectWebSocket>();
    }
}

fn poll_connect_websockets(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ConnectWebSocketTask)>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(response) = future::block_on(future::poll_once(&mut task.task)) {
            let uri = &task.uri;

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
    mut requests: Query<(Entity, &mut ListenWebSocket)>,
    runtime: Res<TokioTasksRuntime>,
) {
    for (entity, mut request) in requests.iter_mut() {
        if request.stream.is_none() {
            warn!("listen websocket missing stream!");
            commands.entity(entity).remove::<ListenWebSocket>();
            continue;
        }

        let uri = request.uri.clone();
        let stream = request.stream.take().unwrap();
        let task = runtime.spawn_background_task(move |mut ctx| async move {
            info!("listening websocket at {}", uri);

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
            .insert(ListenWebSocketTask {
                uri: request.uri.clone(),
                task,
            })
            .remove::<ListenWebSocket>();
    }
}

fn poll_listen_websockets(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ListenWebSocketTask)>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(response) = future::block_on(future::poll_once(&mut task.task)) {
            // TODO: error handling
            let response = response.unwrap();

            // TODO: error handling
            response.unwrap();

            info!("closing websocket connection to {}", task.uri);

            commands.entity(entity).despawn();
        }
    }
}
