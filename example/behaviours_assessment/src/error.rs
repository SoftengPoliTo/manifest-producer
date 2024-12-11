use clap::Error as ClapError;
use manifest_producer_backend::error::Error as BackendError;
use manifest_producer_frontend::error::Error as FrontendError;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub(crate) enum AppError {
    Clap(ClapError),
    Backend(BackendError),
    Frontend(FrontendError),
    Io(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Clap(e) => write!(f, "Argument parsing error: {}", e),
            AppError::Backend(e) => write!(f, "Backend error: {}", e),
            AppError::Frontend(e) => write!(f, "Frontend error: {}", e),
            AppError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl StdError for AppError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            AppError::Clap(e) => Some(e),
            AppError::Backend(e) => Some(e),
            AppError::Frontend(e) => Some(e),
            AppError::Io(e) => Some(e),
        }
    }
}

impl From<ClapError> for AppError {
    fn from(err: ClapError) -> Self {
        AppError::Clap(err)
    }
}

impl From<BackendError> for AppError {
    fn from(err: BackendError) -> Self {
        AppError::Backend(err)
    }
}

impl From<FrontendError> for AppError {
    fn from(err: FrontendError) -> Self {
        AppError::Frontend(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
