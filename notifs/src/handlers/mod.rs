use axum::{debug_handler, extract::ws::WebSocketUpgrade, response::IntoResponse};
use axum_extra::TypedHeader;
use headers::authorization::{Authorization, Bearer};
use tracing::info;
use uuid::Uuid;

use internal::axum::AppError;

use crate::notifs;

#[debug_handler]
pub async fn get_subscribe_notifs(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, AppError> {
    // TODO: validate the server token
    let server_id = Uuid::parse_str(bearer.token())?;

    info!("{} subscribing to notifications ...", server_id);

    Ok(ws.on_upgrade(move |socket| notifs::handle_notifs(socket, server_id)))
}
