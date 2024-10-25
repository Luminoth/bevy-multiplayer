use bevy::prelude::*;
use bevy_mod_reqwest::*;

pub fn find_server<'a>(
    client: &'a mut BevyReqwest,
    player_id: impl AsRef<str>,
) -> BevyReqwestBuilder<'a> {
    info!("finding server ...");

    let url = format!(
        "http://localhost:8080/gameclient/find_server/v1?player_id=\"{}\"",
        player_id.as_ref()
    );

    let req = client.get(url).build().unwrap();

    client.send(req)
}
