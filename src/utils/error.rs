use std::error::Error;
use std::fmt;

/// Define crate-wide error types without using `thiserror`.
#[derive(Debug)]
pub enum AppError {
    DatabaseError(String),
    FileError(String),
    ParseError(String),
    UnexpectedError(String),
    ModelError {
        model_name: String,
        failure_str: String,
    },
}

/// Implement `std::fmt::Display` for `AppError`.
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseError(msg) => write!(f, "Database operation failed: {}", msg),
            AppError::FileError(msg) => write!(f, "File operation failed: {}", msg),
            AppError::ParseError(msg) => write!(f, "Parse operation failed: {}", msg),
            AppError::UnexpectedError(msg) => write!(f, "Unexpected or unknown error: {}", msg),
            AppError::ModelError {
                model_name,
                failure_str,
            } => write!(
                f,
                "Error requesting answer from {}. Error: {}",
                model_name, failure_str
            ),
        }
    }
}

/// Implement `std::error::Error` for `AppError`.
impl Error for AppError {}

/// Custom Result type that uses `AppError`.
pub type Result<T, E = AppError> = std::result::Result<T, E>;
