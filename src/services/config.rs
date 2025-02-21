use anyhow::Result;
use ask_ai::config::AiConfig;
use ask_ai::config::Framework;
use dirs_next::config_dir;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::utils::{error::AppError, logger::log_error};

#[derive(Debug, Deserialize, Serialize)] // Add Serialize to support serialization
pub struct UserConfig {
    pub ai_conf: AiConfig,
    pub theme: String,
    pub ask_conf: bool,
}

/// Get the Legatio configuration directory inside `$HOME/.config/legatio`.
/// Creates the directory if it doesn’t exist.
pub fn get_config_dir() -> Result<PathBuf, AppError> {
    let Some(mut conf_dir) = config_dir() else {
        return Err(AppError::FileError(String::from(
            "Could not find the configuration directory",
        )));
    };

    conf_dir.push("legatio");

    // Create the directory if it doesn't exist
    if !conf_dir.exists() {
        fs::create_dir_all(&conf_dir).map_err(|e| {
            AppError::FileError(format!(
                "Failed to create configuration directory {}: {}",
                &conf_dir.to_string_lossy(),
                e
            ))
        })?;
    }

    Ok(conf_dir)
}

/// Reads the `config.toml` file from the Legatio config directory and parses it into `UserConfig`.
pub fn read_config() -> Result<UserConfig> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.toml");
    // Read the file content as a string
    let toml_content = fs::read_to_string(&config_path).map_err(|e| {
        log_error(&format!(
            "Failed to read file {}: {}",
            &config_path.to_string_lossy(),
            e
        ));
        AppError::FileError(format!(
            "Failed to read file {}: {}",
            &config_path.to_string_lossy(),
            e
        ))
    })?;

    // Parse the TOML content into the UserConfig struct
    let configs: UserConfig = toml::from_str(&toml_content).map_err(|e| {
        log_error(&format!(
            "Failed to parse file {}: {}",
            &config_path.to_string_lossy(),
            e
        ));
        AppError::FileError(format!(
            "Failed to parse file {}: {}",
            &config_path.to_string_lossy(),
            e
        ))
    })?;

    Ok(configs)
}

/// Writes the user config to the `config.toml` file in the Legatio config directory.
pub fn store_config(user_config: &UserConfig) -> Result<()> {
    // Serialize the configs struct into a YAML string
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("config.toml");

    let toml_content =
        toml::to_string(user_config).map_err(|e| AppError::UnexpectedError(e.to_string()))?;

    // Open the file for writing (create or truncate)
    let mut file = File::create(&config_path).map_err(|e| {
        log_error(&format!(
            "Failed to create/open file {}: {}",
            &config_path.to_string_lossy(),
            e
        ));
        AppError::FileError(format!(
            "Failed to create/open file {}: {}",
            &config_path.to_string_lossy(),
            e
        ))
    })?;

    // Write the TOML content to the file
    file.write_all(toml_content.as_bytes()).map_err(|e| {
        log_error(&format!(
            "Failed to write to file {}: {}",
            &config_path.to_string_lossy(),
            e
        ));
        AppError::FileError(format!(
            "Failed to write to file {}: {}",
            &config_path.to_string_lossy(),
            e
        ))
    })?;

    Ok(())
}

pub fn check_config_files() -> Result<(), AppError> {
    let config_dir = get_config_dir()?;

    // Copy default config if missing in config dir
    if !config_dir.join("config.toml").exists() {
        // Default config for user
        let default_config = UserConfig {
            ai_conf: AiConfig {
                llm: Framework::OpenAI,
                model: String::from("chatgpt-4o-latest"),
                max_token: None,
            },
            theme: String::from("Tokyo Storm"),
            ask_conf: true,
        };
        store_config(&default_config).expect("Failed to store default config");
    };

    // Copy default themes if missing in config dir
    if !config_dir.join("themes.toml").exists() {
        let default_themes = PathBuf::from("themes.conf");
        fs::copy(default_themes, config_dir.join("themes.conf"))
            .expect("Failed to store default themes");
    };

    Ok(())
}
