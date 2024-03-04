use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Used for errors that are not due to the user. These could be caused by bugs or system
/// errors like out of disk space or networking issues.
pub struct InternalError(anyhow::Error);

// Tell axum how to convert `InternalError` into a response.
impl IntoResponse for InternalError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal error: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them
// into `Result<_, InternalError>`. That way you don't need to do that manually.
impl<E> From<E> for InternalError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
