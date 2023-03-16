use crate::EmailClient;
use axum::extract::FromRef;
use sqlx::PgPool;
use std::sync::Arc;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct AppState {
    pub connection_pool: PgPool,
    pub email_client: Arc<EmailClient>,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(input: &AppState) -> Self {
        input.connection_pool.clone()
    }
}

impl FromRef<AppState> for Arc<EmailClient> {
    fn from_ref(input: &AppState) -> Self {
        input.email_client.clone()
    }
}
