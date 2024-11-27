use axum::{
    debug_handler,
    extract::{ws::WebSocketUpgrade, State},
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use headers::authorization::{Authorization, Bearer};
use tracing::{error, info};
use uuid::Uuid;

use internal::axum::AppError;

use crate::{notifs, AppState};

#[debug_handler]
pub async fn get_subscribe_notifs(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, AppError> {
    // TODO: validate the server token
    let server_id = Uuid::parse_str(bearer.token())?;

    info!("{} subscribing to notifications ...", server_id);

    let game_servers = app_state.game_servers.clone();

    Ok(ws
        .on_failed_upgrade(move |err| error!("websocket upgrade failed for {}: {}", server_id, err))
        .on_upgrade(move |socket| async move {
            notifs::handle_gameserver_notifs(socket, server_id, game_servers).await;
        }))
}
