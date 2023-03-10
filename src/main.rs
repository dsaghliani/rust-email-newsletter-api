use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::get_configuration;

#[tokio::main]
async fn main() {
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

    sqlx::migrate!()
        .run(&connection_pool)
        .await
        .expect("the migrations should be valid");

    #[allow(clippy::unwrap_used)]
    zero2prod::run(listener).await.unwrap();
}
