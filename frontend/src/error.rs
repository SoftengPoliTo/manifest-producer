use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    ProgressStyleError(indicatif::style::TemplateError),
    MiniJinjaError(minijinja::Error),
}

impl Error {
    pub fn description(&self) -> &str {
        match self {
            Error::Io(_) => "I/O error",
            Error::Json(_) => "JSON serialization error",
            Error::ProgressStyleError(_) => "Progress style error",
            Error::MiniJinjaError(_) => "MiniJinja error",
        }
    }
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
