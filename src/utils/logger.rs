use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

use anyhow::Result;
use chrono::Local;
use log::{Level, Log, Metadata, Record, SetLoggerError};

use crate::services::config::get_config_dir;

/// Custom logger implementation.
struct FileLogger {
    log_file: Mutex<File>, // Wrap the log file in a Mutex for thread-safe access.
}

impl FileLogger {
    /// Creates a new instance of the logger, wrapping around the log file.
    pub fn new(file_path: String) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;
        Ok(FileLogger {
            log_file: Mutex::new(file),
        })
    }
}

impl Log for FileLogger {
    /// Check if the log level is enabled.
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Allow logs up to Trace level, but exclude logs from sqlx
        metadata.level() <= log::Level::Trace && !metadata.target().starts_with("sqlx")
    }

    /// Logs a record.
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Format the log message with a timestamp.
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let log_message = format!("[{}] [{}] {}\n", timestamp, record.level(), record.args());

            // Write the message to the log file.
            if let Ok(mut file) = self.log_file.lock() {
                let _ = file.write_all(log_message.as_bytes());
            }
        }
    }

    /// Flush the log buffer.
    fn flush(&self) {
        if let Ok(mut file) = self.log_file.lock() {
            let _ = file.flush();
        }
    }
}

static LOGGER_READY: std::sync::Once = std::sync::Once::new();
static mut LOGGER: Option<FileLogger> = None;

/// Initialize the logger globally (only in development).
pub async fn initialize_logger() -> Result<()> {
    #[cfg(debug_assertions)] // Only initialize in debug builds
    {
        let config_path = get_config_dir()?.join("logs");
        tokio::fs::create_dir_all(&config_path).await?; // Ensure the log directory exists.
        let timestamp = Local::now().format("log_%Y-%m-%d_%H-%M-%S.log");
        let file_path = config_path.join(timestamp.to_string());

        // Initialize the FileLogger globally.
        unsafe {
            LOGGER_READY.call_once(|| {
                let logger = FileLogger::new(file_path.to_str().unwrap().to_string()).unwrap();
                LOGGER = Some(logger);

                // Set the logger for the global logging facade.
                let _ = log::set_logger(LOGGER.as_ref().unwrap());
                log::set_max_level(log::LevelFilter::Trace);
            });
        }
    }
    Ok(())
}

/// Logs a message with the `INFO` level.
pub fn log_info(message: &str) {
    log::info!("{}", message);
}

/// Logs a message with the `ERROR` level.
pub fn log_error(message: &str) {
    log::error!("{}", message);
}
