use axum::{debug_handler, extract::ws::WebSocketUpgrade, response::IntoResponse};
use axum_extra::TypedHeader;
use headers::authorization::{Authorization, Bearer};
use tracing::info;

use internal::axum::AppError;

use crate::notifs;

#[debug_handler]
pub async fn get_subscribe_notifs(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, AppError> {
    // TODO: this is just hacky stuff until real auth is in
    let server_id = bearer.token().to_owned();

    info!("{} subscribing to notifications ...", server_id);

    Ok(ws.on_upgrade(|socket| notifs::handle_notifs(socket, server_id)))
}
