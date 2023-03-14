use secrecy::{ExposeSecret, Secret};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{net::TcpListener, time::Duration};
use zero2prod::{configuration, telemetry::init_subscriber};

#[tokio::main]
async fn main() {
    init_subscriber();

    let configuration =
        configuration::build().expect("app configuration should be present");
    let listener = bind_listener(
        &configuration.application.host,
        configuration.application.port,
    );

    #[allow(clippy::unwrap_used)]
    let connection_pool =
        create_connection_pool(&configuration.database.connection_string())
            .unwrap();

    #[allow(clippy::unwrap_used)]
    zero2prod::run(listener, connection_pool).await.unwrap();
}

fn bind_listener(host: &str, port: u16) -> TcpListener {
    let address = format!("{host}:{port}");
    TcpListener::bind(address).expect("the provided address should be valid")
}

fn create_connection_pool(
    connection_string: &Secret<String>,
) -> sqlx::Result<PgPool> {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .connect_lazy(connection_string.expose_secret())
}
