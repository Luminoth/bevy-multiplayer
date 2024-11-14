use bevy::prelude::*;
use bevy_mod_reqwest::*;

use common::user::UserId;

const HOST: &str = "http://localhost:8000";

pub fn find_server<'a>(
    client: &'a mut BevyReqwest,
    user_id: UserId,
) -> anyhow::Result<BevyReqwestBuilder<'a>> {
    info!("finding server ...");

    let url = format!("{}/gameclient/find_server/v1", HOST);

    let req = client
        .get(url)
        // TODO: should be auth JWT token
        .bearer_auth(user_id.to_string())
        .build()?;

    Ok(client.send(req))
}
