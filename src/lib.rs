#![allow(clippy::unused_async)]

use anyhow::anyhow;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Form, Router, Server,
};
use config::{Config, ConfigError};
use serde::Deserialize;
use sqlx::PgPool;
use std::net::TcpListener;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

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
        .layer(TraceLayer::new_for_http())
        .with_state(connection_pool)
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
    State(connection_pool): State<PgPool>,
    Form(subscription_data): Form<SubscriptionData>,
) -> impl IntoResponse {
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name)
        VALUES ($1, $2, $3)
        "#,
        Uuid::new_v4(),
        subscription_data.email,
        subscription_data.name
    )
    .execute(&connection_pool)
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(error) => {
            eprintln!("Failed to execute query: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
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
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    #[must_use]
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}
