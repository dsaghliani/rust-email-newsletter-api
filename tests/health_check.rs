#![allow(clippy::unwrap_used)]

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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("the provided address should be valid");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener);

    tokio::spawn(server);

    format!("http://127.0.0.1:{port}")
}
