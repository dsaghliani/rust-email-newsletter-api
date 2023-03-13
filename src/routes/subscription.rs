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

#[axum::debug_handler]
pub async fn subscribe(
    State(connection_pool): State<PgPool>,
    Form(subscription_data): Form<SubscriptionData>,
) -> impl IntoResponse {
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
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(error) => {
            error!("Failed to execute query: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
