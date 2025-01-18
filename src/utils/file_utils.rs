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

pub fn read_file(file_path: &str, project_id: &str) -> Result<Scroll> {
    let content = fs::read_to_string(file_path)?;
    Ok(Scroll::new(file_path, &content, project_id))
}
