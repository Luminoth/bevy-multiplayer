use bevy::prelude::*;
use bevy_mod_reqwest::*;

use crate::server::GameServerInfo;

pub fn heartbeat(mut client: BevyReqwest, info: &GameServerInfo) {
    info!("heartbeat");

    let url = "http://localhost:8080/gameserver/heartbeat/v1";

    let req = client
        .post(url)
        .json(&common::gameserver::GameServerInfo {
            server_id: info.server_id.clone(),
            game_session_id: None,
        })
        .build()
        .unwrap();

    client
        .send(req)
        .on_response(|req: Trigger<ReqwestResponseEvent>| {
            let req = req.event();
            let res = req.as_str();
            info!("return data: {res:?}");
        })
        .on_error(|trigger: Trigger<ReqwestErrorEvent>| {
            let e = &trigger.event().0;
            error!("error: {e:?}");
        });
}
