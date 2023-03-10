use sqlx::PgPool;
use std::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use zero2prod::get_configuration;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zero2prod=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let configuration =
        get_configuration().expect("app configuration should be present");
    let listener = {
        let address = format!("127.0.0.1:{}", configuration.application_port);
        TcpListener::bind(address).expect("the provided address should be valid")
    };
    let connection_pool = {
        let connection_string = configuration.database.connection_string();

        #[allow(clippy::unwrap_used)]
        PgPool::connect(&connection_string).await.unwrap()
    };

    // sqlx::migrate!()
    //     .run(&connection_pool)
    //     .await
    //     .expect("the migrations should be valid");

    #[allow(clippy::unwrap_used)]
    zero2prod::run(listener, connection_pool).await.unwrap();
}
