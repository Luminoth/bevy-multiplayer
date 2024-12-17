use axum::{routing::get, Router};

use crate::{handlers::gameserver::*, state::AppState};

pub fn init_routes(app: Router<AppState>) -> Router<AppState> {
    app.route("/gameserver/notifs/v1", get(get_subscribe_notifs))
}
