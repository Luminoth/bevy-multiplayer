use axum::{debug_handler, extract::Query, Json};
use axum_extra::TypedHeader;
use headers::authorization::{Authorization, Bearer};
use serde::Deserialize;

use common::{gameclient::*, user::User};
use internal::axum::AppError;

#[derive(Debug, Deserialize)]
pub struct FindServerParamsV1 {}

#[debug_handler]
pub async fn get_find_server_v1(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    Query(_params): Query<FindServerParamsV1>,
) -> Result<Json<FindServerResponseV1>, AppError> {
    let user = User::read_from_token(bearer.token()).await?;

    println!("TODO: find server for {}", user.user_id);

    Ok(Json(FindServerResponseV1 {
        address: "127.0.0.1".into(),
        port: 5576,
    }))
}
