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
    // TODO: validate the client token
    let user_id = Uuid::parse_str(bearer.token())?;

    info!("{} subscribing to notifications ...", user_id);

    let game_clients = app_state.game_clients.clone();

    Ok(ws
        .on_failed_upgrade(move |err| error!("websocket upgrade failed for {}: {}", user_id, err))
        .on_upgrade(move |socket| async move {
            notifs::handle_gameclient_notifs(socket, user_id, game_clients).await;
        }))
}
