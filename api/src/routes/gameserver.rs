use axum::{routing::post, Router};

use crate::{handlers::gameserver::*, state::AppState};

pub fn init_routes(app: Router<AppState>) -> Router<AppState> {
    app.route("/gameserver/heartbeat/v1", post(post_heartbeat_v1))
}
