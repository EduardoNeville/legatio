use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::sync::Mutex;

// Global static variable for the log file, wrapped in an `Option<Mutex<File>>`
static mut LOG_FILE: Option<Mutex<File>> = None;

/// Initializes the logger with the specified log file path.
/// Should be called once before logging any messages.
pub fn initialize_logger(file_path: &str) -> io::Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(file_path)?;

    // Initialize the global LOG_FILE variable
    unsafe {
        LOG_FILE = Some(Mutex::new(file));
    }
    Ok(())
}

/// Logs a message to the log file with the given level (e.g., "INFO" or "ERROR").
fn log_message(level: &str, message: &str) {
    let log_message = format!("[{}] {}\n", level, message);

    // Access the global LOG_FILE and write to it
    unsafe {
        if let Some(ref mutex) = LOG_FILE {
            if let Ok(mut file) = mutex.lock() {
                let _ = file.write_all(log_message.as_bytes());
            }
        } else {
            eprintln!("Logger not initialized. Please call `initialize_logger` first.");
        }
    }
}

/// Logs an info message.
pub fn log_info(message: &str) {
    log_message("INFO", message);
}

/// Logs an error message.
pub fn log_error(message: &str) {
    log_message("ERROR", message);
}
