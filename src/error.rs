use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("Invalid format file")]
    InvalidElf { #[from] source: goblin::error::Error },

    #[error("Capstone error: {0}")]
    Capstone(String),

    #[error("No Debug-info")]
    DebugInfo,

    #[error("No API found")]
    APIListEmpty,
    
    #[error(".text section not found")]
    TextSectionNotFound,

    #[error(".plt section not found")]
    PLTSectionNotFound,

    #[error("Demangling error")]
    DemanglingError(#[from] cpp_demangle::error::Error),

    #[error("Formatting error: {0}")]
    FormattingError(#[from] std::fmt::Error),

    #[error("JSON serialization error")]
    Json(#[from] serde_json::Error),

    #[error("Gimli error")]
    GimliError(#[from] gimli::Error),

    #[error("Object error")]
    ObjectError(#[from] object::Error),

    #[error("Prefix not found")]
    PrefixNotFound,

}

pub type Result<T> = ::std::result::Result<T, Error>;
