#[cfg(test)]
mod tests {
    use legatio::services::config::*;
    use legatio::utils::logger::*;
    use std::fs::{self, File};
    use std::io::{BufRead, BufReader};
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    use anyhow::Result;

    /// Helper function to simulate log output and validate results.
    #[tokio::test]
    async fn test_logger_functionality() -> Result<()> {
        // Step 1: Initialize the logger.
        initialize_logger().await?;

        // Step 2: Fetch the log directory and file created by the logger.
        let config_path = get_config_dir()?.join("logs");

        // Check the log directory was created.
        assert!(config_path.exists());

        // Step 3: Log some messages at various log levels.
        log_info("This is an info log for testing.");
        log_error("This is an error log for testing.");
        log::warn!("This is a warning log for testing.");

        // Allow time for logs to be written asynchronously.
        thread::sleep(Duration::from_millis(200));

        // Step 4: Retrieve the most recent log file created in the directory.
        let log_file = get_latest_log_file(&config_path)?;
        assert!(log_file.exists(), "Log file was not created.");

        // Step 5: Verify the log file contents.
        let file = File::open(&log_file)?;
        let reader = BufReader::new(file);

        let mut lines = Vec::new();
        for line in reader.lines() {
            lines.push(line?);
        }

        // Validate that log entries exist.
        assert!(
            lines
                .iter()
                .any(|line| line.contains("INFO")
                    && line.contains("This is an info log for testing."))
        );
        assert!(lines.iter().any(
            |line| line.contains("ERROR") && line.contains("This is an error log for testing.")
        ));
        assert!(lines.iter().any(
            |line| line.contains("WARN") && line.contains("This is a warning log for testing.")
        ));

        Ok(())
    }

    /// Helper function to fetch the most recent log file from a directory.
    fn get_latest_log_file(log_dir: &PathBuf) -> Result<PathBuf> {
        let mut log_files = fs::read_dir(log_dir)?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .collect::<Vec<_>>();

        // Sort by modification time, descending, to get the latest file.
        log_files.sort_by_key(|entry| {
            entry
                .metadata()
                .and_then(|meta| meta.modified())
                .map(|mtime| {
                    std::time::SystemTime::now()
                        .duration_since(mtime)
                        .unwrap_or_default()
                })
                .ok()
        });

        match log_files.first() {
            Some(log_file) => Ok(log_file.path()),
            None => Err(anyhow::anyhow!("No log files found.")),
        }
    }
}
