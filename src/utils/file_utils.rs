use std::fs;
use anyhow::Result;
use walkdir::WalkDir;

pub fn get_all_files_in_directory(dir: &str) -> Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        files.push(entry.path().display().to_string());
    }
    Ok(files)
}

pub fn read_files(file_paths: &[String]) -> Result<Vec<(String, String)>> {
    let mut files = Vec::new();
    for path in file_paths {
        let content = fs::read_to_string(path)?;
        files.push((path.clone(), content));
    }
    Ok(files)
}
