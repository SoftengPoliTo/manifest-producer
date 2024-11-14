use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    InvalidFormat(goblin::error::Error),
    InvalidFileName,
    Capstone(capstone::Error),
    DebugInfo,
    LangNotFound,
    TextSectionNotFound,
    DemanglingError(cpp_demangle::error::Error),
    FormattingError(std::fmt::Error),
    Json(serde_json::Error),
    GimliError(gimli::Error),
    ObjectError(object::Error),
    FunctionNotFound(String),
    ProgressStyleError(indicatif::style::TemplateError),
}

impl Error {
    pub fn description(&self) -> &str {
        match self {
            Error::Io(_) => "I/O error",
            Error::InvalidFormat(_) => "Invalid format file",
            Error::InvalidFileName => "No valid file name",
            Error::Capstone(_) => "Capstone error",
            Error::DebugInfo => "No Debug-info",
            Error::LangNotFound => "Language not found",
            Error::TextSectionNotFound => ".text section not found",
            Error::DemanglingError(_) => "Demangling error",
            Error::FormattingError(_) => "Formatting error",
            Error::Json(_) => "JSON serialization error",
            Error::GimliError(_) => "Gimli error",
            Error::ObjectError(_) => "Object error",
            Error::FunctionNotFound(_) => "Function not found",
            Error::ProgressStyleError(_) => "Progress style error",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error -> {}", e),
            Error::InvalidFormat(msg) => write!(f, "Invalid format file -> {}", msg),
            Error::InvalidFileName => write!(f, "No valid file name"),
            Error::Capstone(e) => write!(f, "Capstone error -> {}", e),
            Error::DebugInfo => write!(f, "No debug info available"),
            Error::LangNotFound => write!(f, "Programming language not found"),
            Error::TextSectionNotFound => write!(f, "'.text' section not found"),
            Error::DemanglingError(e) => write!(f, "Demangling error -> {}", e),
            Error::FormattingError(e) => write!(f, "Formatting error -> {}", e),
            Error::Json(e) => write!(f, "JSON serialization error -> {}", e),
            Error::GimliError(e) => write!(f, "Gimli error -> {}", e),
            Error::ObjectError(e) => write!(f, "Object error -> {}", e),
            Error::FunctionNotFound(func) => write!(f, "Function '{}' not found", func),
            Error::ProgressStyleError(e) => write!(f, "Progress style error -> {}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::InvalidFormat(e) => Some(e),
            Error::Capstone(e) => Some(e),
            Error::DemanglingError(e) => Some(e),
            Error::FormattingError(e) => Some(e),
            Error::Json(e) => Some(e),
            Error::GimliError(e) => Some(e),
            Error::ObjectError(e) => Some(e),
            Error::ProgressStyleError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<goblin::error::Error> for Error {
    fn from(err: goblin::error::Error) -> Self {
        Error::InvalidFormat(err)
    }
}

impl From<capstone::Error> for Error {
    fn from(err: capstone::Error) -> Self {
        Error::Capstone(err)
    }
}

impl From<cpp_demangle::error::Error> for Error {
    fn from(err: cpp_demangle::error::Error) -> Self {
        Error::DemanglingError(err)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(err: std::fmt::Error) -> Self {
        Error::FormattingError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl From<gimli::Error> for Error {
    fn from(err: gimli::Error) -> Self {
        Error::GimliError(err)
    }
}

impl From<object::Error> for Error {
    fn from(err: object::Error) -> Self {
        Error::ObjectError(err)
    }
}

impl From<indicatif::style::TemplateError> for Error {
    fn from(err: indicatif::style::TemplateError) -> Self {
        Error::ProgressStyleError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
