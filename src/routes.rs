pub mod health_check {
    use axum::{http::StatusCode, response::IntoResponse};

    pub async fn health() -> impl IntoResponse {
        StatusCode::OK
    }
}

pub mod subscription;
