#![allow(clippy::module_name_repetitions)]

use crate::{domain::NewSubscriber, extractors::ValidatedForm};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip_all,
    fields(
        subscriber_email = %subscription_data.email.as_ref(),
        subscriber_name = %subscription_data.name.as_ref(),
    )
)]
pub async fn subscribe(
    State(connection_pool): State<PgPool>,
    ValidatedForm(subscription_data): ValidatedForm<NewSubscriber>,
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
    subscription_data: &NewSubscriber,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name)
        VALUES ($1, $2, $3)
        "#,
        Uuid::new_v4(),
        subscription_data.email.as_ref(),
        subscription_data.name.as_ref()
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
