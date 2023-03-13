use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{configuration, telemetry::init_subscriber};

#[tokio::main]
async fn main() {
    init_subscriber();

    let configuration =
        configuration::build().expect("app configuration should be present");
    let listener = bind_listener(configuration.application_port);

    #[allow(clippy::unwrap_used)]
    let connection_pool = create_connection_pool(
        configuration.database.connection_string().expose_secret(),
    )
    .await
    .unwrap();

    #[allow(clippy::unwrap_used)]
    zero2prod::run(listener, connection_pool).await.unwrap();
}

fn bind_listener(port: u16) -> TcpListener {
    let address = format!("127.0.0.1:{port}");
    TcpListener::bind(address).expect("the provided address should be valid")
}

async fn create_connection_pool(connection_string: &str) -> sqlx::Result<PgPool> {
    PgPool::connect(connection_string).await
}
