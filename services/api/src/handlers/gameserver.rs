use axum::{debug_handler, extract::State, Json};
use axum_extra::TypedHeader;
use headers::authorization::{Authorization, Bearer};
use uuid::Uuid;

use common::gameserver::*;
use internal::axum::AppError;

use crate::{gameservers, gamesessions, models, state::AppState};

#[debug_handler]
pub async fn post_heartbeat_v1(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(mut app_state): State<AppState>,
    Json(request): Json<PostHeartbeatRequestV1>,
) -> Result<Json<PostHeartbeatResponseV1>, AppError> {
    // TODO: validate the server token
    let server_id = Uuid::parse_str(bearer.token())?;

    let gameserver_info = models::gameserver::GameServerInfo::new(server_id, &request.server_info);
    let game_session_info =
        request
            .server_info
            .game_session_info
            .as_ref()
            .map(|game_session_info| {
                models::gamesession::GameSessionInfo::new(server_id, game_session_info)
            });

    let mut pipeline = redis::pipe();

    gameservers::update_gameserver(&mut pipeline, &gameserver_info).await?;
    if let Some(game_session_info) = game_session_info {
        gamesessions::update_game_session(&mut pipeline, &game_session_info).await?;
    }

    let _: () = pipeline
        .query_async(&mut app_state.redis_connection)
        .await?;

    Ok(Json(PostHeartbeatResponseV1 {}))
}
