#![allow(clippy::unused_async)]

pub mod configuration;
pub mod telemetry;

mod routes;

use anyhow::{anyhow, Context};
use axum::{
    routing::{get, post},
    Router, Server,
};
use routes::{health_check::health, subscription::subscribe};
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
        .context("failed to run the migrations from `zero2prod::run`")?;

    let router = build_router(connection_pool);

    Server::from_tcp(listener)
        .context("couldn't create the server from the provided `TcpListener`")?
        .serve(router.into_make_service())
        .await
        .context("something went wrong running the server")?;
        })?;

    Ok(())
}

fn build_router(connection_pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/subscriptions", post(subscribe))
        .layer(TraceLayer::new_for_http().make_span_with(RequestIdMakeSpan::new()))
        .with_state(connection_pool)
}
