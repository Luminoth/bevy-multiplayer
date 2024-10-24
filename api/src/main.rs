mod error;
mod handlers;
mod http_tracing;
mod options;
mod redis;
mod routes;
mod state;
mod util;

use std::net::SocketAddr;

use axum::{
    http::{HeaderValue, Method},
    middleware, Router,
};
use clap::Parser;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnFailure, TraceLayer},
    LatencyUnit,
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use options::Options;
use state::AppState;

fn init_logging() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

fn init_cors_layer() -> anyhow::Result<CorsLayer> {
    info!("Initializing CORS layer...");

    let layer = CorsLayer::new()
        .allow_methods([Method::OPTIONS, Method::HEAD, Method::GET, Method::POST])
        .allow_origin("*".parse::<HeaderValue>()?);

    Ok(layer)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = Options::parse();

    init_logging()?;

    let redis_connection_pool = redis::connect("redis://localhost").await?;

    let app_state = AppState::new(options, redis_connection_pool);

    let app = routes::init_routes(Router::new())
        .layer(init_cors_layer()?)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(http_tracing::tracing_wrapper))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            DefaultMakeSpan::new()
                                //.level(Level::INFO)
                                .include_headers(true),
                        )
                        //.on_request(http_tracing::on_request)
                        //.on_response(http_tracing::on_response),
                        .on_failure(DefaultOnFailure::new().latency_unit(LatencyUnit::Micros)),
                )
                .into_inner(),
        )
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("listening on {}", listener.local_addr()?);
    Ok(axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?)
}
