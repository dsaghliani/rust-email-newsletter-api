#![allow(clippy::unused_async)]

use anyhow::anyhow;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Form, Router, Server,
};
use config::{Config, ConfigError};
use serde::Deserialize;
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
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

#[derive(Deserialize)]
struct SubscriptionData {
    name: String,
    email: String,
}

#[axum::debug_handler]
async fn subscribe(
    Form(subscription_data): Form<SubscriptionData>,
) -> impl IntoResponse {
    StatusCode::OK
}

/// Load the configuration for the app.
///
/// # Errors
///
/// Will return an error when:
///
/// - the specified configuration file cannot be found;
/// - the loaded configuration cannot be deserialized into [`Settings`].
pub fn get_configuration() -> Result<Settings, ConfigError> {
    // `config` will look for a file named, "configuration," in the top-level
    // directory with any extension it knows how to parse: yaml, json, toml, etc.
    Config::builder()
        .add_source(config::File::with_name("configuration"))
        .build()?
        .try_deserialize()
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub application_port: u16,
}
