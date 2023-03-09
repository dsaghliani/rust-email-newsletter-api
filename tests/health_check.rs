#![allow(clippy::unwrap_used)]

use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::run;

#[tokio::test]
async fn health_check_works() {
    // Arrange.
    let address = spawn_app();
    let address = format!("{address}/health_check");
    let client = reqwest::Client::new();

    // Act.
    let response = client.get(&address).send().await.unwrap();

    // Assert.
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[sqlx::test]
async fn subscribe_returns_200_for_valid_form_data(pool: PgPool) {
    // Arrange.
    let address = spawn_app();
    let address = format!("{address}/subscriptions");
    let client = reqwest::Client::new();

    // Act.
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&address)
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
        .expect("the table should have at least 1 entry");

    assert_eq!("ursula_le_guin@gmail.com", saved.email);
    assert_eq!("le guin", saved.name);
}

#[tokio::test]
async fn subscribe_returns_422_when_data_is_missing() {
    // Arrange.
    let address = spawn_app();
    let address = format!("{address}/subscriptions");
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both the name and the email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act.
        let response = client
            .post(&address)
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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("the provided address should be valid");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener);

    tokio::spawn(server);

    format!("http://127.0.0.1:{port}")
}
