use bb8_redis::redis::AsyncCommands;
use tokio::time::Duration;
use tracing::info;

use internal::notifs::Notification;

use crate::AppState;

pub async fn notify_gameserver(
    app_state: &AppState,
    notification: Notification,
    _ttl: Option<Duration>,
) -> anyhow::Result<()> {
    let notif = serde_json::to_string(&notification)?;
    info!("notifying gameserver: {}", notif);

    let mut conn = app_state.redis_connection_pool.get().await?;

    let _: () = conn
        .publish(internal::GAMESERVER_NOTIFS_CHANNEL, notif)
        .await?;

    // TODO: add the notif the recipient's mailbox (with ttl if provided)

    Ok(())
}
