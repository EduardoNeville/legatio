use std::fs;
use anyhow::Result;
use crate::utils::structs::Scroll;
use std::path::PathBuf;

const IGNORE_LIST: &[&str] = &[
    "node_modules",
    "target", // Common build output directory for Rust projects
    ".git",
    ".DS_Store",
    "*.log",  // Example: ignore log files
    "/hidden_folder", // Example: ignore a specific folder
];

pub fn get_contents(
    dir: &str,
    dir_or_file: bool,
    max_depth: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    let mut current_level = vec![(PathBuf::from(dir), 0)]; // Store path with current depth

    while !current_level.is_empty() {
        let mut next_level = Vec::new();

        for (current_dir, depth) in current_level {
            // Check if we have reached the max search depth
            if depth >= max_depth {
                continue;
            }

            // Read directory entries
            let entries = fs::read_dir(&current_dir)?;

            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                // Get the file name or directory name as a string
                if let Some(name) = path.file_name().and_then(|f| f.to_str()) {
                    // Skip items in the ignore list
                    if IGNORE_LIST.iter().any(|&ignore| name == ignore || path.ends_with(ignore)) {
                        continue;
                    }
                }

                if dir_or_file && path.is_dir() {
                    // If looking for directories, add them to the result and schedule for visiting
                    result.push(path.display().to_string());
                    next_level.push((path, depth + 1));
                } else if !dir_or_file && path.is_file() {
                    // If looking for files, add them to the result
                    result.push(path.display().to_string());
                }
            }
        }

        current_level = next_level;
    }

    Ok(result)
}


pub fn read_file(file_path: &str, project_id: &str) -> Result<Scroll> {
    let content = fs::read_to_string(file_path)?;
    Ok(Scroll::new(file_path, &content, project_id))
}
