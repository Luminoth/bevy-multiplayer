use axum::{routing::get, Router};

use crate::{handlers::gameclient::*, state::AppState};

pub fn init_routes(app: Router<AppState>) -> Router<AppState> {
    app.route("/gameclient/notifs/v1", get(get_subscribe_notifs))
}
