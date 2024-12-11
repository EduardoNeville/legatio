use std::fs;
use rayon::prelude::*;
use anyhow::Result;
use ignore::WalkBuilder;
use crate::utils::structs::File;

// List of directories and file patterns to ignore
const IGNORE_LIST: &[&str] = &[
    "node_modules",
    "target", // Common build output directory for Rust projects
    ".git",
    ".DS_Store",
    "*.log",  // Example: ignore log files
    "/hidden_folder", // Example: ignore a specific folder
];

pub fn get_contents(dir: String, dir_or_file: bool) -> Result<Vec<String>> {
    let mut result = Vec::new();
    let mut current_level = vec![dir];

    while !current_level.is_empty() {
        let mut next_level = Vec::new();

        for current_dir in current_level {
            let walker = WalkBuilder::new(current_dir.clone())
                .max_depth(Some(1))
                .hidden(false)
                .standard_filters(false)
                .add_custom_ignore_filename(".myignore")
                .filter_entry(|e| !IGNORE_LIST.iter().any(|p| e.path().ends_with(p)))
                .build();

            for entry in walker {
                let entry = entry?;

                let path = entry.path();
                let path_str = path.display().to_string();

                if !result.contains(&current_dir.clone()){
                    result.push(current_dir.clone())
                }

                if path_str != current_dir.clone() {
                    if path.is_dir(){
                        if next_level.contains(&path_str){
                            next_level.retain(|x| x != &path_str);
                        } else {
                            next_level.push(path.display().to_string());
                        }
                    }

                    if if dir_or_file { path.is_dir() } else { path.is_file() }{
                        result.push(path.display().to_string())
                    }
                }
            }
        }
        current_level = next_level;
    }

    Ok(result)
}


pub fn read_files(file_paths: &[String]) -> Result<Vec<File>> {
    file_paths.par_iter()
        .map(|path| {
            let content = fs::read_to_string(path)?;
            Ok(File::new(&path.clone(), &content))
        })
        .collect()
}
