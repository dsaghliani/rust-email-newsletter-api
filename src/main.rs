use newsletter::{configuration, create_email_client, telemetry::init_subscriber};
use sqlx::postgres::PgPoolOptions;
use std::{net::TcpListener, time::Duration};
use tracing::debug;

#[tokio::main]
async fn main() {
    init_subscriber(std::io::stdout);

    let configuration =
        configuration::build().expect("app configuration should be present");

    debug!("Detected the following configuration: {configuration:?}");

    let listener = bind_listener(
        &configuration.application.host,
        configuration.application.port,
    );

    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(10))
        .connect_lazy_with(configuration.database.connect_options());

    let email_client = create_email_client(&configuration);

    #[allow(clippy::unwrap_used)]
    newsletter::run(listener, connection_pool, email_client)
        .await
        .unwrap();
}

fn bind_listener(host: &str, port: u16) -> TcpListener {
    let address = format!("{host}:{port}");
    TcpListener::bind(address).expect("the provided address should be valid")
}
