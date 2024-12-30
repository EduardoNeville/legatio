use anyhow::Result;
use crate::utils::structs::File;

pub async fn construct_system_prompt(files: &[File]) -> Result<String> {
    let system_prompt = files.iter()
        .map(|file| {
            let file_name = file.file_path.rsplit('/').next().unwrap_or(""); // Handles empty paths safely
            format!("```{:?}\n{:?}```\n", file_name, file.content)
        })
        .collect::<Vec<_>>()
        .join(""); // Joining avoids intermediate allocations with push_str
    
    Ok(system_prompt)
}
