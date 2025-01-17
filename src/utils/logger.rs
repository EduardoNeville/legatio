use log::{info, error};
use chrono::Local;

pub fn initialize_logger(log_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    fern::Dispatch::new()
        // Set the log level for output
        .level(log::LevelFilter::Info)
        // Format the log output
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        // Log to the specified file
        .chain(fern::log_file(log_file)?)
        // Apply the configuration
        .apply()?;

    Ok(())
}

// Logs an info message
pub fn log_info(message: &str) {
    info!("{}", message);
}

// Logs an error message
pub fn log_error(message: &str) {
    error!("{}", message);
}
