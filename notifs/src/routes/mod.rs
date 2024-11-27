mod gameclient;
mod gameserver;

use axum::Router;
use tracing::info;

use internal::axum as axum_util;

use crate::state::AppState;

pub fn init_routes(app: Router<AppState>) -> Router<AppState> {
    info!("initializing routes...");

    // TODO: this is ugly
    let app = gameclient::init_routes(app);
    let app = gameserver::init_routes(app);

    app.fallback(axum_util::handler_404)
}
