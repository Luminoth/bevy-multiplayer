use axum::{debug_handler, extract::Query, Json};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct FindServerParamsV1 {
    player_id: String,
}

#[derive(Debug, Serialize)]
pub struct FindServerResponseV1 {
    pub address: String,
    pub port: u16,
}

#[debug_handler]
pub async fn get_find_server_v1(
    Query(params): Query<FindServerParamsV1>,
) -> Result<Json<FindServerResponseV1>, AppError> {
    println!("TODO: find server for {}", params.player_id);

    Ok(Json(FindServerResponseV1 {
        address: "127.0.0.1".into(),
        port: 5576,
    }))
}
