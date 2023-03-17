use newsletter::{
    configuration, create_email_client, run, telemetry::init_subscriber,
};
use once_cell::sync::Lazy;
use sqlx::PgPool;
use std::net::TcpListener;

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        init_subscriber(std::io::stdout);
    } else {
        init_subscriber(std::io::sink);
    }
});

pub struct TestApp {
    pub address: String,
}

pub fn spawn_app(connection_pool: PgPool) -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("the provided address should be valid");
    let port = listener.local_addr().unwrap().port();
    let email_client = {
        let configuration =
            configuration::build().expect("app configuration should be present");
        create_email_client(&configuration)
    };

    let server = run(listener, connection_pool, email_client);

    tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{port}"),
    }
}
