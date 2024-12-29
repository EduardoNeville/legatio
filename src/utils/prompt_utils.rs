use anyhow::Result;
use crate::utils::structs::File;

pub async fn construct_system_prompt(files: &[File]) -> Result<String> {
    let mut system_prompt = String::new();
    for (_idx, file) in files.iter().enumerate() {
        system_prompt.push_str(
            &format!("```{:?}\n{:?}```\n",
                file.file_path.split("/").last().unwrap(),
                file.content,
            )
        );
    }

    Ok(system_prompt)
}


