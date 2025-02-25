mod error;
mod http_tracing;

use std::fmt;
use std::net::SocketAddr;

use axum::{
    body::Bytes, debug_handler, extract::ConnectInfo, http::StatusCode, http::Uri,
    response::IntoResponse,
};
use http::{Request, header::AsHeaderName};
use http_body_util::BodyExt;
use tracing::{debug, info};

pub use error::*;
pub use http_tracing::*;

#[debug_handler]
pub async fn handler_404(uri: Uri) -> impl IntoResponse {
    debug!("invalid resource: {}", uri);

    (StatusCode::NOT_FOUND, "Resource not found")
}

// copied from warp's log filter
pub struct OptFmt<T>(pub Option<T>);

impl<T: fmt::Display> fmt::Display for OptFmt<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref t) = self.0 {
            fmt::Display::fmt(t, f)
        } else {
            f.write_str("-")
        }
    }
}

pub fn get_request_header<B, K>(request: &Request<B>, header: K) -> Option<&str>
where
    K: AsHeaderName,
{
    let header = request.headers().get(header);
    if let Some(header) = header {
        if let Ok(header) = header.to_str() {
            return Some(header);
        }
    }
    None
}

fn get_forwarded_for<B>(request: &Request<B>) -> Option<&str> {
    // TODO: not sure if header::FORWARDED works here or not
    let forwarded_for = get_request_header(request, "X-Forwarded-For");
    if let Some(forwarded_for) = forwarded_for {
        let addrs = forwarded_for.split(',').collect::<Vec<&str>>();
        if !addrs.is_empty() {
            return Some(addrs[0]);
        }
    }
    None
}

pub fn get_forwarded_addr<B>(request: &Request<B>) -> Option<SocketAddr> {
    let forwarded_for = get_forwarded_for(request);
    if let Some(forwarded_for) = forwarded_for {
        let port = if let Some(remote_addr) = request.extensions().get::<ConnectInfo<SocketAddr>>()
        {
            remote_addr.port()
        } else {
            0
        };

        return format!("{}:{}", forwarded_for, port).parse().ok();
    }

    None
}

pub async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        info!("{direction} body = {body:?}");
    }

    Ok(bytes)
}
