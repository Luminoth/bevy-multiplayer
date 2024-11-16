use axum::extract::ws::{Message, WebSocket};
use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use tracing::info;
use uuid::Uuid;

use crate::state::GameServerSet;

pub type NotifSender = SplitSink<WebSocket, Message>;

async fn idle_notifs(mut receiver: SplitStream<WebSocket>) {
    // idle on the receiver until the connection is closed
    while let Some(Ok(_)) = receiver.next().await {
        // ignore whatever we received
    }
}

pub async fn handle_notifs(socket: WebSocket, server_id: Uuid, game_servers: GameServerSet) {
    info!("{} subscribed to notifications ...", server_id);

    let (sender, receiver) = socket.split();
    game_servers.write().await.insert(server_id, sender);

    idle_notifs(receiver).await;

    info!("{} closed notifications connection", server_id);

    game_servers.write().await.remove(&server_id);
}
