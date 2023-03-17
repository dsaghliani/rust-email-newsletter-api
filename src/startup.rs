use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use axum::{
    routing::{get, post},
    Router, Server,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use crate::{
    configuration::Settings,
    email_client::EmailClient,
    routes::{health_check::health, subscription::subscribe},
    telemetry::RequestIdMakeSpan,
    AppState,
};

pub struct App {
    port: u16,
    address: SocketAddr,
    connection_pool: PgPool,
    email_client: EmailClient,
    listener: TcpListener,
}

impl App {
    pub const fn port(&self) -> u16 {
        self.port
    }

    // NOTE: This only works when `build_app()` uses `connect_lazy()`. Otherwise,
    // a connection would already have been made by the time this function is
    // called.
    pub fn set_custom_connection_pool(&mut self, pool: PgPool) {
        self.connection_pool = pool;
    }

    /// Run the contained hyper server.
    ///
    /// # Errors
    ///
    /// Will return an error if:
    ///
    /// - cannot create a `hyper::Server`;
    /// - something goes wrong while running the server.
    pub async fn run(self) -> hyper::Result<()> {
        let Self {
            connection_pool,
            email_client,
            listener,
            address,
            ..
        } = self;
        let router = build_router(connection_pool, email_client);

        info!("Listening on {address}");

        Server::from_tcp(listener)?
            .serve(router.into_make_service())
            .await?;

        Ok(())
    }
}

/// Build the app with the given configuration.
///
/// # Errors
///
/// Will return an error if:
///
/// - cannot bind the `TcpListener` used to start the server;
/// - cannot run the migrations in the `migrations/` directory;
/// - cannot get the local address of the `TcpListener` (for logging purposes);
/// - something goes wrong running the server.
pub async fn build_app(configuration: Settings) -> anyhow::Result<App> {
    // Create the `TcpListener`.
    let listener = {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        TcpListener::bind(address).context("couldn't bind the `TcpListener`")?
    };

    // Create a Postgres connection pool.
    // NOTE: `connect_lazy` is crucial here! It's what lets the `spawn_app()` test
    // helper override the returned app's connection pool with the test one it got
    // from `#[sqlx::test]`. If you're going to change it, consider how you're
    // going to rework `App::set_custom_connection_pool`, too.
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(10))
        .connect_lazy_with(configuration.database.connect_options());

    // Run all migrations.
    run_migrations(&connection_pool)
        .await
        .context("something went wrong running the migrations")?;

    // Create the email client.
    let email_client = create_email_client(&configuration);

    // Extract the address.
    let address = listener
        .local_addr()
        .context("couldn't get the `TcpListener`'s local address")?;

    Ok(App {
        port: address.port(),
        address,
        connection_pool,
        email_client,
        listener,
    })
}

#[tracing::instrument(name = "Running migrations")]
async fn run_migrations(connection_pool: &PgPool) -> sqlx::Result<()> {
    sqlx::migrate!()
        .run(connection_pool)
        .await
        // In the future, when it has stabilized, this can be replaced with
        // `Result::inspect_err`.
        .map_err(|error| {
            error!("Failed to run migrations: {error}");
            error
        })?;

    Ok(())
}

fn create_email_client(configuration: &Settings) -> EmailClient {
    let sender_email = configuration
        .email_client
        .sender()
        .expect("the sender email should be valid");
    let base_url = configuration.email_client.base_url.clone();
    let auth_token = configuration.email_client.authorization_token.clone();
    let timeout = configuration.email_client.timeout();

    EmailClient::new(sender_email, base_url, auth_token, timeout)
}

fn build_router(connection_pool: PgPool, email_client: EmailClient) -> Router {
    let email_client = Arc::new(email_client);

    Router::new()
        .route("/health", get(health))
        .route("/subscriptions", post(subscribe))
        .layer(TraceLayer::new_for_http().make_span_with(RequestIdMakeSpan::new()))
        .with_state(AppState {
            connection_pool,
            email_client,
        })
}
