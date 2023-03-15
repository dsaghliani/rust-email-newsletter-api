use sqlx::postgres::PgPoolOptions;
use std::{net::TcpListener, time::Duration};
use tracing::debug;
use zero2prod::{configuration, telemetry::init_subscriber};

#[tokio::main]
async fn main() {
    init_subscriber();

    let configuration =
        configuration::build().expect("app configuration should be present");

    debug!("Detected the following configuration: {configuration:?}");

    let listener = bind_listener(
        &configuration.application.host,
        configuration.application.port,
    );

    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(30))
        .connect_lazy_with(configuration.database.connect_options());

    #[allow(clippy::unwrap_used)]
    zero2prod::run(listener, connection_pool).await.unwrap();
}

fn bind_listener(host: &str, port: u16) -> TcpListener {
    let address = format!("{host}:{port}");
    TcpListener::bind(address).expect("the provided address should be valid")
}
