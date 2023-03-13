#![allow(clippy::unwrap_used)]

use once_cell::sync::Lazy;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{run, telemetry::init_subscriber};

static TRACING: Lazy<()> = Lazy::new(init_subscriber);

struct TestApp {
    address: String,
}

#[sqlx::test]
async fn health_check_works(pool: PgPool) {
    // Arrange.
    let app = spawn_app(pool);
    let endpoint = format!("{}/health_check", app.address);
    let client = reqwest::Client::new();

    // Act.
    let response = client.get(&endpoint).send().await.unwrap();

    // Assert.
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[sqlx::test]
async fn subscribe_returns_200_for_valid_form_data(pool: PgPool) {
    // Arrange.
    let app = spawn_app(pool.clone());
    let endpoint = format!("{}/subscriptions", app.address);
    let client = reqwest::Client::new();

    // Act.
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&endpoint)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .unwrap();

    // Assert.
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&pool)
        .await
        .expect("the entry should've been inserted");

    assert_eq!("ursula_le_guin@gmail.com", saved.email);
    assert_eq!("le guin", saved.name);
}

#[sqlx::test]
async fn subscribe_returns_422_when_data_is_missing(pool: PgPool) {
    // Arrange.
    let app = spawn_app(pool);
    let endpoint = format!("{}/subscriptions", app.address);
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both the name and the email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act.
        let response = client
            .post(&endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .unwrap();

        // Assert.
        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with `422: Bad Request` when the payload was: \
            {error_message}"
        );
    }
}

fn spawn_app(connection_pool: PgPool) -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("the provided address should be valid");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener, connection_pool);

    tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{port}"),
    }
}
