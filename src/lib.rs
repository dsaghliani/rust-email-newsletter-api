#![allow(clippy::unused_async)]

use anyhow::anyhow;
use axum::{
    http::StatusCode, response::IntoResponse, routing::get, Router, Server,
};
use std::net::TcpListener;

/// Run the server.
///
/// # Errors
///
/// Propagates the error if:
///
/// - the server can't be created from the provided `TcpListener`;
/// - something goes wrong when running the server.
pub async fn run(listener: TcpListener) -> anyhow::Result<()> {
    let router = build_router();

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

fn build_router() -> Router {
    Router::new().route("/health_check", get(health_check))
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
