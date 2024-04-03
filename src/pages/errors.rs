use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use core::fmt;

pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self.0.downcast_ref::<ValidationError>() {
            Some(err) => err.to_response(),
            None => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal error: {}", self.0),
            )
                .into_response(),
        }
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them
// into `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
// ---------------------------------------------------------------------------------------

/// Used for errors that are not due to the user. These could be caused by bugs or system
/// errors like out of disk space or networking issues.
#[derive(Debug)]
pub struct InternalError {
    mesg: String,
}

// impl InternalError {
//     pub fn new(message: &str) -> InternalError {
//         InternalError {
//             mesg: message.to_owned(),
//         }
//     }
// }

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.mesg)
    }
}

impl std::error::Error for InternalError {}
// ---------------------------------------------------------------------------------------

/// Used when the user tries to do something illegal, e.g. use a negative weight.
#[derive(Debug)]
pub struct ValidationError {
    mesg: String,
}

impl ValidationError {
    pub fn new(message: &str) -> ValidationError {
        ValidationError {
            mesg: message.to_owned(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.mesg)
    }
}

impl std::error::Error for ValidationError {}

impl ValidationError {
    fn to_response(&self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            format!("Validation error: {}", self.mesg),
        )
            .into_response()
    }
}
