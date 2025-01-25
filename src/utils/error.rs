use thiserror::Error;

/// Define crate-wide error types using `thiserror`.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database operation failed: {0}")]
    DatabaseError(String),

    #[error("File operation failed: {0}")]
    FileError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Unexpected or unknown error: {0}")]
    UnexpectedError(String),
}

/// Custom Result type that uses `AppError`.
pub type Result<T, E = AppError> = std::result::Result<T, E>;
