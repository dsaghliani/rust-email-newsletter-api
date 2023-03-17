use sqlx::PgPool;

use crate::helpers::spawn_app;

#[sqlx::test]
async fn subscribe_returns_200_for_valid_form_data(pool: PgPool) {
    // Arrange.
    let app = spawn_app(pool.clone()).await;

    // Act.
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = app.post_subscriptions(body.into()).await;

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
    let app = spawn_app(pool).await;
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both the name and the email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act.
        let response = app.post_subscriptions(invalid_body.into()).await;

        // Assert.
        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with `422: Bad Request` when the payload was: \
            {error_message}"
        );
    }
}

#[sqlx::test]
async fn subscribe_returns_422_when_fields_are_present_but_invalid(pool: PgPool) {
    // Arrange.
    let app = spawn_app(pool).await;
    let test_cases = vec![
        (
            "name=&email=ursula_le_guin%40gmail.com",
            "the name is empty",
        ),
        ("name=le%20guin&email=", "the email is empty"),
        (
            "name=le%20guin&email=definitely-not-an-email",
            "the email is invalid",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act.
        let response = app.post_subscriptions(invalid_body.into()).await;

        // Assert.
        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not return a `422: Unprocessable Entity` when the payload was: \
            {error_message}"
        );
    }
}
