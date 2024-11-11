use axum::{routing::get, Router};
use tracing::info;

use internal::axum as axum_util;

use crate::{handlers::*, state::AppState};

pub fn init_routes(app: Router<AppState>) -> Router<AppState> {
    info!("initializing routes...");

    let app = app.route("/notifs/v1", get(get_subscribe_notifs));

    app.fallback(axum_util::handler_404)
}
