use axum::Router;
use tracing::info;

use internal::axum as axum_util;

use crate::state::AppState;

pub fn init_routes(app: Router<AppState>) -> Router<AppState> {
    info!("initializing routes...");

    app.fallback(axum_util::handler_404)
}
