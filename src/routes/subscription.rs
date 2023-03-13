#![allow(clippy::module_name_repetitions)]

use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{error, info_span, Instrument};
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
        request_id = %Uuid::new_v4(),
        subscriber_email = %subscription_data.email,
        subscriber_name = %subscription_data.name,
    )
)]
pub async fn subscribe(
    State(connection_pool): State<PgPool>,
    Form(subscription_data): Form<SubscriptionData>,
) -> impl IntoResponse {
    let query_span = info_span!("Saving new subscriber details in the database");

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name)
        VALUES ($1, $2, $3)
        "#,
        Uuid::new_v4(),
        subscription_data.email,
        subscription_data.name
    )
    .execute(&connection_pool)
    .instrument(query_span)
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(error) => {
            error!("Failed to execute query: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
