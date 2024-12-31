use log::{info, error};

// Logs an info message
pub fn log_info(message: &str) {
    info!("{}", message);
}

// Logs an error message
pub fn log_error(message: &str) {
    error!("{}", message);
}
