use std::error::Error as StdError;
use std::fmt;

/// Custom error type for the `frontend` crate.
///
/// This enum encapsulates various error types that may occur in the frontend module:
/// - `Io`: Errors related to file input/output operations.
/// - `Json`: Errors encountered during JSON serialization or deserialization.
/// - `ProgressStyleError`: Errors in styling or formatting progress indicators using the `indicatif` crate.
/// - `MiniJinjaError`: Errors raised by the `minijinja` templating engine.
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    ProgressStyleError(indicatif::style::TemplateError),
    MiniJinjaError(minijinja::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error -> {}", e),
            Error::Json(e) => write!(f, "JSON serialization error -> {}", e),
            Error::ProgressStyleError(e) => write!(f, "Progress style error -> {}", e),
            Error::MiniJinjaError(e) => write!(f, "MiniJinja error -> {}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Json(e) => Some(e),
            Error::ProgressStyleError(e) => Some(e),
            Error::MiniJinjaError(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl From<indicatif::style::TemplateError> for Error {
    fn from(err: indicatif::style::TemplateError) -> Self {
        Error::ProgressStyleError(err)
    }
}

impl From<minijinja::Error> for Error {
    fn from(err: minijinja::Error) -> Self {
        Error::MiniJinjaError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
