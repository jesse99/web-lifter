use axum::{
    http::{uri::InvalidUri, StatusCode},
    response::{IntoResponse, Response},
};
use handlebars::RenderError;
use std::fmt::Display;

pub enum Error {
    /// User tried to input something erroneous. Note that front end validation should
    /// catch many of these but custom tools or bad actors can easily bypass that.
    ValidationError(String),

    /// User action failed to complete because of something that should have been
    /// impossible.
    InternalError(String),
    // /// Used for things like disk full.
    // RuntimeError(String), // if we need to switch on the actual err we can add a ner variant for that or box the original error
}

#[macro_export]
macro_rules! validation_err {
    ($($t:tt)*) => {Err(Error::ValidationError(format!($($t)*)))}
}

#[macro_export]
macro_rules! internal_err {
    ($($t:tt)*) => {Err(Error::InternalError(format!($($t)*)))}
}

pub trait Unwrapper<T> {
    /// Unwrap or return an error.
    fn unwrap_or_err(self, message: &str) -> Result<T, Error>;
}

impl<T> Unwrapper<T> for Option<T> {
    fn unwrap_or_err(self, message: &str) -> Result<T, Error> {
        match self {
            Some(ok) => Ok(ok),
            None => Err(Error::InternalError(message.to_string())),
        }
    }
}

impl<T, E> Unwrapper<T> for Result<T, E>
where
    E: Display,
{
    fn unwrap_or_err(self, message: &str) -> Result<T, Error> {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error::InternalError(format!("{e}: {message}"))),
        }
    }
}

/// Convert our Error into an Axum response.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let err = match self {
            Error::ValidationError(s) => format!("Validation Error: {s}"),
            Error::InternalError(s) => format!("Internal Error: {s}"),
            // Error::RuntimeError(s) => format!("Runtime Error: {s}"),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, err).into_response()
    }
}

impl From<RenderError> for Error {
    fn from(item: RenderError) -> Self {
        Error::InternalError(item.to_string())
    }
}

impl From<InvalidUri> for Error {
    fn from(item: InvalidUri) -> Self {
        Error::InternalError(item.to_string())
    }
}
