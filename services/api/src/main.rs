mod gameservers;
mod gamesessions;
mod handlers;
mod models;
mod notifs;
mod options;
mod routes;
mod state;

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

use internal::{axum as axum_util, redis};

use options::Options;
use state::AppState;

// TODO: add authentication

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
    info!("initializing CORS layer...");

    let layer = CorsLayer::new()
        .allow_methods([Method::OPTIONS, Method::HEAD, Method::GET, Method::POST])
        .allow_origin("*".parse::<HeaderValue>()?);

    Ok(layer)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = Options::parse();

    init_logging()?;

    let redis_connection = redis::connect(options.redis_host.clone()).await?;

    let app_state = AppState::new(options, redis_connection);

    let addr = app_state
        .options
        .address()
        .parse::<SocketAddr>()
        .unwrap_or_else(|_| panic!("Invalid address: {}", app_state.options.address()));

    let app = routes::init_routes(Router::new())
        .layer(init_cors_layer()?)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(axum_util::tracing_wrapper))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            DefaultMakeSpan::new()
                                //.level(Level::INFO)
                                .include_headers(true),
                        )
                        //.on_request(axum_util::on_request)
                        //.on_response(axum_util::on_response)
                        .on_failure(DefaultOnFailure::new().latency_unit(LatencyUnit::Micros)),
                )
                .into_inner(),
        )
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("listening on {}", listener.local_addr()?);
    Ok(axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?)
}
