#![allow(clippy::unused_async)]

pub mod configuration;
pub mod telemetry;

mod routes;

use anyhow::anyhow;
use axum::{
    routing::{get, post},
    Router, Server,
};
use routes::{health_check::health_check, subscription::subscribe};
use sqlx::PgPool;
use std::net::TcpListener;
use telemetry::RequestIdMakeSpan;
use tower_http::trace::TraceLayer;

/// Run the server.
///
/// # Errors
///
/// Propagates the error if:
///
/// - the server can't be created from the provided `TcpListener`;
/// - something goes wrong when running the server.
pub async fn run(
    listener: TcpListener,
    connection_pool: PgPool,
) -> anyhow::Result<()> {
    sqlx::migrate!()
        .run(&connection_pool)
        .await
        .expect("the migrations should be valid");

    let router = build_router(connection_pool);

    Server::from_tcp(listener)
        .map_err(|err| {
            anyhow!(
                "Couldn't create the server from the provided `TcpListener`: {err}"
            )
        })?
        .serve(router.into_make_service())
        .await
        .map_err(|err| {
            anyhow!("Something went wrong running the server: {err}")
        })?;

    Ok(())
}

fn build_router(connection_pool: PgPool) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(TraceLayer::new_for_http().make_span_with(RequestIdMakeSpan::new()))
        .with_state(connection_pool)
}
