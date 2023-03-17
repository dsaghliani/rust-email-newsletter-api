use sqlx::PgPool;

use crate::helpers::spawn_app;

#[sqlx::test]
async fn health_works(pool: PgPool) {
    // Arrange.
    let app = spawn_app(pool);
    let endpoint = format!("{}/health", app.address);
    let client = reqwest::Client::new();

    // Act.
    let response = client.get(&endpoint).send().await.unwrap();

    // Assert.
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
