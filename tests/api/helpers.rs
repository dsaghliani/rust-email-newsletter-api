use newsletter::{build_app, configuration, telemetry::init_subscriber};
use once_cell::sync::Lazy;
use sqlx::PgPool;

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

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        let endpoint = format!("{}/subscriptions", self.address);

        reqwest::Client::new()
            .post(endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("sending the request should not fail")
    }
}

pub async fn spawn_app(connection_pool: PgPool) -> TestApp {
    Lazy::force(&TRACING);

    let mut configuration = configuration::build()
        .expect("app configuration should be present and valid");
    configuration.application.port = 0;

    let mut app = build_app(configuration).await.unwrap();
    app.set_custom_connection_pool(connection_pool);

    let port = app.port();

    tokio::spawn(async { app.run().await });

    TestApp {
        address: format!("http://127.0.0.1:{port}"),
    }
}
