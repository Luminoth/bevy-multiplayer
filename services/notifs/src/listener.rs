use axum::extract::ws::Message;
use futures_util::{SinkExt, StreamExt};
use tokio::task;
use tracing::{debug, info};
use uuid::Uuid;

use internal::notifs::Notification;

use crate::AppState;

/*
TODO:

A problem with this approach is that every notifier instance
will receieve every notification, even if it's not intended for it

A resolution to this could be to have a separate channel for each
notifier and maintain a set of game servers to notifiers to look up
where to send the notifications. This also gives some agency
to the sender because it can know sooner in the flow if
the recipient is available or not
 */

pub async fn start_gameclient_listener(
    app_state: &AppState,
) -> anyhow::Result<task::JoinHandle<()>> {
    info!("starting game client notifs listener ...");

    let client = redis::Client::open(app_state.options.redis_host.as_str())?;
    let (mut sink, mut stream) = client.get_async_pubsub().await?.split();
    sink.subscribe(internal::GAMECLIENT_NOTIFS_CHANNEL)
        .await
        .unwrap();

    let game_clients = app_state.game_clients.clone();
    Ok(task::spawn(async move {
        // TODO: error handling
        loop {
            let msg = stream.next().await.unwrap();
            let payload: String = msg.get_payload().unwrap();
            debug!(
                "got game client notif: {} (channel: {})",
                payload,
                msg.get_channel_name()
            );

            let notif: Notification = serde_json::from_str(&payload).unwrap();
            let recipient = Uuid::parse_str(&notif.recipient).unwrap();

            {
                let mut game_clients = game_clients.write().await;
                if let Some(sender) = game_clients.get_mut(&recipient) {
                    info!("notifying game client {}", recipient);
                    sender.send(Message::Text(payload)).await.unwrap();
                } else {
                    debug!("ignoring notif for {}", recipient);
                }
            }
        }
    }))
}

pub async fn start_gameserver_listener(
    app_state: &AppState,
) -> anyhow::Result<task::JoinHandle<()>> {
    info!("starting game server notifs listener ...");

    let client = redis::Client::open(app_state.options.redis_host.as_str())?;
    let (mut sink, mut stream) = client.get_async_pubsub().await?.split();
    sink.subscribe(internal::GAMESERVER_NOTIFS_CHANNEL)
        .await
        .unwrap();

    let game_servers = app_state.game_servers.clone();
    Ok(task::spawn(async move {
        // TODO: error handling
        loop {
            let msg = stream.next().await.unwrap();
            let payload: String = msg.get_payload().unwrap();
            debug!(
                "got game server notif: {} (channel: {})",
                payload,
                msg.get_channel_name()
            );

            let notif: Notification = serde_json::from_str(&payload).unwrap();
            let recipient = Uuid::parse_str(&notif.recipient).unwrap();

            {
                let mut game_servers = game_servers.write().await;
                if let Some(sender) = game_servers.get_mut(&recipient) {
                    info!("notifying game server {}", recipient);
                    sender.send(Message::Text(payload)).await.unwrap();
                } else {
                    debug!("ignoring notif for {}", recipient);
                }
            }
        }
    }))
}
