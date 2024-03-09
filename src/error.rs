use thiserror::Error;

/// Custom error type for manifest-producer.
///
/// This error type encompasses various kinds of errors that can occur during the execution of manifest-producer.
#[derive(Debug, Error)]
pub enum Error {
    /// An I/O error occurred.
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    /// The file has an invalid format.
    #[error("Invalid format file")]
    InvalidElf {
        #[from]
        source: goblin::error::Error,
    },

    /// An error occurred in the Capstone disassembly library.
    #[error("Capstone error: {0}")]
    Capstone(String),

    /// No debug information was found.
    #[error("No Debug-info")]
    DebugInfo,

    /// No API was found.
    #[error("No API found")]
    APIListEmpty,

    /// The `.text` section was not found.
    #[error(".text section not found")]
    TextSectionNotFound,

    /// The `.plt` section was not found.
    #[error(".plt section not found")]
    PLTSectionNotFound,

    /// An error occurred while demangling symbols.
    #[error("Demangling error")]
    DemanglingError(#[from] cpp_demangle::error::Error),

    /// A formatting error occurred.
    #[error("Formatting error: {0}")]
    FormattingError(#[from] std::fmt::Error),

    /// An error occurred during JSON serialization.
    #[error("JSON serialization error")]
    Json(#[from] serde_json::Error),

    /// A Gimli error occurred.
    #[error("Gimli error")]
    GimliError(#[from] gimli::Error),

    /// An error occurred related to object handling.
    #[error("Object error")]
    ObjectError(#[from] object::Error),

    /// The prefix was not found.
    #[error("Prefix not found")]
    PrefixNotFound,
}

/// A specialized `Result` type for manifest-producer.
pub type Result<T> = ::std::result::Result<T, Error>;
