use bevy::prelude::*;
use bevy_mod_reqwest::*;

const HOST: &str = "http://localhost:8000";

pub fn find_server<'a>(
    client: &'a mut BevyReqwest,
    player_id: impl AsRef<str>,
) -> BevyReqwestBuilder<'a> {
    info!("finding server ...");

    let url = format!(
        "{}/gameclient/find_server/v1?player_id=\"{}\"",
        HOST,
        player_id.as_ref()
    );

    let req = client.get(url).build().unwrap();

    client.send(req)
}
