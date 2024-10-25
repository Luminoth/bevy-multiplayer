use bevy::prelude::*;
use bevy_mod_reqwest::*;

#[allow(dead_code)]
pub fn find_server(mut client: BevyReqwest, player_id: impl AsRef<str>) {
    println!("info finding server ...");

    let url = format!(
        "http://localhost:8080/gameclient/find_server/v1?player_id=\"{}\"",
        player_id.as_ref()
    );

    let req = client.get(url).build().unwrap();

    client
        .send(req)
        .on_response(|req: Trigger<ReqwestResponseEvent>| {
            let req = req.event();
            let res = req.as_str();
            println!("return data: {res:?}");

            // TODO: ok but now what?
        })
        .on_error(|trigger: Trigger<ReqwestErrorEvent>| {
            let e = &trigger.event().0;
            error!("find server error: {:?}", e);

            // TODO: ok but now what?
        });
}
