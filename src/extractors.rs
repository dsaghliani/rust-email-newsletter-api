pub use validated_form::ValidatedForm;

mod validated_form {
    use async_trait::async_trait;
    use axum::{
        extract::{rejection::FormRejection, FromRequest},
        http::{Request, StatusCode},
        response::{IntoResponse, Response},
        Form,
    };
    use serde::de::DeserializeOwned;
    use thiserror::Error;
    use tracing::error;
    use validator::{Validate, ValidationErrors};

    #[derive(Debug, Clone, Copy, Default)]
    pub struct ValidatedForm<T>(pub T);

    #[async_trait]
    impl<T, S, B> FromRequest<S, B> for ValidatedForm<T>
    where
        T: DeserializeOwned + Validate,
        S: Send + Sync,
        Form<T>: FromRequest<S, B, Rejection = FormRejection>,
        B: Send + 'static,
    {
        type Rejection = Error;

        #[tracing::instrument(name = "Validating form input", skip_all)]
        async fn from_request(
            request: Request<B>,
            state: &S,
        ) -> Result<Self, Self::Rejection> {
            let Form(value) = Form::<T>::from_request(request, state)
                .await
                .map_err(|error| {
                    error!("Failed to serialize the form: {error}");
                    error
                })?;

            value.validate().map_err(|errors| {
                error!("Failed to validate the form: [{errors}]");
                errors
            })?;

            Ok(Self(value))
        }
    }

    #[derive(Debug, Error)]
    pub enum Error {
        #[error(transparent)]
        AxumFormRejection(#[from] FormRejection),
        #[error(transparent)]
        ValidationError(#[from] ValidationErrors),
    }

    impl IntoResponse for Error {
        fn into_response(self) -> Response {
            match self {
                Self::ValidationError(_) => {
                    let message = format!("Input validation error: [{self}]")
                        .replace('\n', ", ");
                    (StatusCode::UNPROCESSABLE_ENTITY, message)
                }
                Self::AxumFormRejection(_) => {
                    (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
                }
            }
            .into_response()
        }
    }
}
