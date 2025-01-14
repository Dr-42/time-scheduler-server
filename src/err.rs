use std::fmt::{self, Display};

use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};

#[macro_export]
macro_rules! err_from_type {
    ($error_type:expr) => {
        Error {
            error_type: $error_type,
            file: file!(),
            line: line!(),
            column: column!(),
            additional: None,
        }
    };
    ($error_type:expr, $($args:tt)*) => {
        Error {
            error_type: $error_type,
            file: file!(),
            line: line!(),
            column: column!(),
            additional: Some(format!($($args)*)),
        }
    }
}

#[macro_export]
macro_rules! err_with_context {
    ($err:expr) => {
        Error::from(($err, file!(), line!(), column!(), None))
    };
    ($err:expr, $($args:tt)*) => {
        Error::from((
            $err,
            file!(),
            line!(),
            column!(),
            Some(format!($($args)*)),
        ))
    };
}

pub enum ErrorType {
    AxumError(axum::http::Error),
    SerdeError(serde_json::Error),
    Tokio(tokio::io::Error),
    Chrono,
    IdenticalBlockType,
    InternalRustError,
}

impl Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::AxumError(error) => write!(f, "Axum error: {}", error),
            ErrorType::SerdeError(error) => write!(f, "Serde error: {}", error),
            ErrorType::Tokio(error) => write!(f, "Tokio error: {}", error),
            ErrorType::Chrono => write!(f, "Chrono error"),
            ErrorType::IdenticalBlockType => write!(f, "Blocktypes Identical"),
            ErrorType::InternalRustError => write!(f, "Internal Rust error"),
        }
    }
}

pub struct Error {
    pub error_type: ErrorType,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
    pub additional: Option<String>,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ERROR: {}:{}:{} : {}",
            self.file, self.line, self.column, self.error_type
        )?;
        if let Some(additional) = &self.additional {
            write!(f, "\n\tAdditional: {}", additional)?;
        }
        Ok(())
    }
}

impl IntoResponse for Error {
    #[allow(clippy::unwrap_used)]
    fn into_response(self) -> Response<Body> {
        let status_code = StatusCode::INTERNAL_SERVER_ERROR;
        Response::builder()
            .status(status_code)
            .body(Body::from(self.to_string()))
            .unwrap()
    }
}

impl<E> From<(E, &'static str, u32, u32, Option<String>)> for Error
where
    E: Into<ErrorType>,
{
    fn from(
        (err, file, line, column, additional): (E, &'static str, u32, u32, Option<String>),
    ) -> Self {
        Error {
            error_type: err.into(),
            file,
            line,
            column,
            additional,
        }
    }
}

// Conversions for error types
impl From<axum::http::Error> for ErrorType {
    fn from(err: axum::http::Error) -> Self {
        ErrorType::AxumError(err)
    }
}

impl From<serde_json::Error> for ErrorType {
    fn from(err: serde_json::Error) -> Self {
        ErrorType::SerdeError(err)
    }
}

impl From<tokio::io::Error> for ErrorType {
    fn from(err: tokio::io::Error) -> Self {
        ErrorType::Tokio(err)
    }
}

impl From<chrono::OutOfRangeError> for ErrorType {
    fn from(_err: chrono::OutOfRangeError) -> Self {
        ErrorType::Chrono
    }
}

impl From<chrono::ParseError> for ErrorType {
    fn from(_err: chrono::ParseError) -> Self {
        ErrorType::Chrono
    }
}
