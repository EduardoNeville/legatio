use std::{fs::File, io::Write};
use std::result::Result::Ok;
use std::fs;
use std::io::ErrorKind;

use crate::utils::logger::log_error;

pub fn store_app_state(app_state: &str) {
    let app_state = serde_json::to_string(&app_state).unwrap();
    let mut file = File::create("config.json").expect("Could not create file!");
    file.write(app_state.as_bytes()).unwrap();
}


pub async fn get_app_state() -> Result<()> {
    // Attempt to read the `config.json` file
    let file_content = match fs::read_to_string("config.json") {
        Ok(content) => content,
        Err(error) => {
            // Log the error if the file is missing or cannot be read
            if error.kind() == ErrorKind::NotFound {
                log_error("Error: `config.json` file not found.");
            } else {
                log_error(&format!("Error: Failed to read `config.json` file: {}", error));
            }
            return Err(()); // Return an empty tuple in case of failure
        },
    };

    // Attempt to parse JSON into AppState
    let app_state = match serde_json::from_str(&file_content) {
        Ok(state) => state,
        Err(error) => {
            log_error(&format!("Error: Failed to parse `config.json`: {}", error));
            return Err(()); // Return an empty tuple in case of parsing failure
        },
    };

    Ok(app_state)
}

