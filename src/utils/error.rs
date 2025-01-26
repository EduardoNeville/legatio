use std::error::Error;
use std::fmt;

/// Define crate-wide error types without using `thiserror`.
#[derive(Debug)]
pub enum AppError {
    DatabaseError(String),
    FileError(String),
    ValidationError(String),
    UnexpectedError(String),
}

/// Implement `std::fmt::Display` for `AppError`.
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseError(msg) => write!(f, "Database operation failed: {}", msg),
            AppError::FileError(msg) => write!(f, "File operation failed: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::UnexpectedError(msg) => write!(f, "Unexpected or unknown error: {}", msg),
        }
    }
}

/// Implement `std::error::Error` for `AppError`.
impl Error for AppError {}

/// Custom Result type that uses `AppError`.
pub type Result<T, E = AppError> = std::result::Result<T, E>;
