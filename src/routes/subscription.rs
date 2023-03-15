#![allow(clippy::module_name_repetitions)]

use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SubscriptionData {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(connection_pool, subscription_data),
    fields(
        subscriber_email = %subscription_data.email,
        subscriber_name = %subscription_data.name,
    )
)]
pub async fn subscribe(
    State(connection_pool): State<PgPool>,
    Form(subscription_data): Form<SubscriptionData>,
) -> impl IntoResponse {
    if (insert_subscriber(&connection_pool, &subscription_data).await).is_ok() {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip_all
)]
async fn insert_subscriber(
    connection_pool: &PgPool,
    subscription_data: &SubscriptionData,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name)
        VALUES ($1, $2, $3)
        "#,
        Uuid::new_v4(),
        subscription_data.email,
        subscription_data.name
    )
    .execute(connection_pool)
    .await
    // In the future, when it has stabilized, this can be replaced with
    // `Result::inspect_err`.
    .map_err(|error| {
        error!("Faild to execute query: {error}");
        error
    })?;

    Ok(())
}
