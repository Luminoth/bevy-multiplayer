mod gameserver;

use axum::Router;
use tracing::info;

use crate::handlers;
use crate::state::AppState;

pub fn init_routes(app: Router<AppState>) -> Router<AppState> {
    info!("initializing routes...");

    // TODO: this is ugly
    let app = gameserver::init_routes(app);

    app.fallback(handlers::handler_404)
}
