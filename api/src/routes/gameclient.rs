use axum::{routing::get, Router};

use crate::{handlers::gameclient::*, state::AppState};

pub fn init_routes(app: Router<AppState>) -> Router<AppState> {
    app.route("/gameclient/find_server/v1", get(get_find_server_v1))
}
