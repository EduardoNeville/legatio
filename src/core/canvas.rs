use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use anyhow::{Result, Context};
use crate::utils::logger::log_error;
use crate::{
    core::prompt::prompt_chain,
    utils::structs::{Prompt, Project},
};

/// Writes a chain of prompts and their outputs into the canvas file (`legatio.md`). 
/// If any errors occur, they will be logged and propagated.
///
/// # Parameters:
/// - `project`: The `Project` containing the `legatio.md` file path.
/// - `prompts`: An optional slice of all `Prompt` objects in the project.
/// - `prompt`: An optional reference to a `Prompt` object at the head of the chain.
///
/// # Returns:
/// - `Ok(())` if the canvas file is updated successfully.
/// - Logs an error and propagates it otherwise.
pub fn chain_into_canvas(
    project: &Project,
    prompts: Option<&[Prompt]>,
    prompt: Option<&Prompt>,
) -> Result<()> {
    // Construct the file path for `legatio.md`
    let file_path = PathBuf::from(&project.project_path).join("legatio.md");

    // Open the file for writing, truncating its content
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)  // Create the file if it doesn't exist
        .open(&file_path)
        .with_context(|| format!("Failed to open canvas file {:?}", file_path))?;

    // Proceed only if both prompts and a single prompt are provided
    if let (Some(prompts), Some(prompt)) = (prompts, prompt) {
        // Generate the chain
        let mut chain = prompt_chain(prompts, prompt);
        chain.reverse();

        // Write each prompt and its output to the file
        for prompt in chain {
            if let Err(err) = writeln!(file, "# PROMPT {}\n{}", prompt.prompt_id, prompt.content) {
                log_error(&format!("Failed to write prompt to canvas file: {:?}", err));
                return Err(err.into());
            }
            if let Err(err) = writeln!(file, "# OUTPUT {}\n{}", prompt.prompt_id, prompt.output) {
                log_error(&format!("Failed to write output to canvas file: {:?}", err));
                return Err(err.into());
            }
        }
    }

    // Add the placeholder section for asking the model below
    if let Err(err) = writeln!(file, "\n# ASK MODEL BELLOW") {
        log_error(&format!("Failed to write ASK MODEL section to canvas file: {:?}", err));
        return Err(err.into());
    }

    Ok(())
}

/// Checks if the canvas file (`legatio.md`) contains the correct sequences of prompts
/// and retrieves unmatched content appearing after the `# ASK MODEL BELLOW` section.
///
/// Logs and propagates any file-reading errors.
///
/// # Parameters:
/// - `project`: The `Project` containing the `legatio.md` file path.
///
/// # Returns:
/// - `Ok(String)` containing unmatched content after the `# ASK MODEL BELLOW` marker if present.
/// - Logs and propagates any errors otherwise.
pub fn chain_match_canvas(project: &Project) -> Result<String> {
    // Construct the file path for `legatio.md`
    let canvas_path = PathBuf::from(&project.project_path).join("legatio.md");

    // Read the entire content of the canvas file
    let canvas = fs::read_to_string(&canvas_path)
        .with_context(|| format!("Failed to read canvas file at {:?}", canvas_path))?;

    // Find the index of the `# ASK MODEL BELLOW` marker
    if let Some(match_index) = canvas.find("# ASK MODEL BELLOW") {
        // Return everything after the marker as the unmatched content
        let unmatched_content_start = canvas[match_index + "# ASK MODEL BELLOW".len()..].to_string();
        return Ok(unmatched_content_start);
    }

    // If the marker is not found, log and return an empty string
    log_error(&format!("The canvas file does not include the '# ASK MODEL BELLOW' marker."));
    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::fs::File;
    use std::io::Write;

    use tokio;
    use anyhow::Result;

    use super::*;
    use crate::utils::structs::{Prompt, Project};

    fn setup_temp_project() -> Result<(Project, PathBuf)> {
        // Create a temporary directory for the project
        let temp_dir = tempfile::tempdir()?;
        let project_path = temp_dir.path().to_path_buf();

        // Create a new Project
        let project = Project::new(project_path.to_string_lossy().as_ref());

        // Return the project and the temporary directory path (to keep the temp_dir alive)
        Ok((project, project_path))
    }

    #[tokio::test]
    async fn test_chain_into_canvas() -> Result<()> {
        // Setup a temp project
        let (project, project_path) = setup_temp_project()?;

        // Create some test Prompts
        let prompt1 = Prompt::new("proj_id_1", "Describe AI", "AI is awesome", "prev_id_1");
        let prompt2 = Prompt::new("proj_id_1", "Explain Rust", "Rust is fast", &prompt1.prompt_id);

        let prompts = vec![prompt1.clone(), prompt2.clone()];

        // Invoke chain_into_canvas
        chain_into_canvas(&project, Some(&prompts), Some(&prompt2))?;

        // Check the contents of the canvas file
        let canvas_path = project_path.join("legatio.md");
        let canvas_content = fs::read_to_string(&canvas_path)?;

        // Verify the expected content
        assert!(canvas_content.contains("# PROMPT"));
        assert!(canvas_content.contains(&prompt1.content));
        assert!(canvas_content.contains(&prompt2.content));
        assert!(canvas_content.contains("AI is awesome"));
        assert!(canvas_content.contains("Rust is fast"));
        assert!(canvas_content.contains("# ASK MODEL BELLOW"));

        Ok(())
    }

    #[tokio::test]
    async fn test_chain_match_canvas() -> Result<()> {
        // Setup a temp project
        let (project, project_path) = setup_temp_project()?;

        // Create a test canvas file
        let canvas_path = project_path.join("legatio.md");
        let mut file = File::create(&canvas_path)?;

        let canvas_content = r#"
# PROMPT prompt_id_1
Describe AI
# OUTPUT prompt_id_1
AI is awesome

# PROMPT prompt_id_2
Explain Rust
# OUTPUT prompt_id_2
Rust is fast

# ASK MODEL BELLOW
Write about programming languages.
"#;

        file.write_all(canvas_content.as_bytes())?;
        drop(file); // Flush and drop the file

        // Invoke chain_match_canvas
        let unmatched_content = chain_match_canvas(&project)?;

        // Verify the unmatched content
        assert!(unmatched_content.contains("Write about programming languages."));
        assert!(!unmatched_content.contains("# PROMPT"));
        assert!(!unmatched_content.contains("# OUTPUT"));

        Ok(())
    }

    #[tokio::test]
    async fn test_chain_into_canvas_with_no_prompts() -> Result<()> {
        // Setup a temp project
        let (project, project_path) = setup_temp_project()?;

        // Invoke chain_into_canvas with no prompts and no single prompt
        chain_into_canvas(&project, None, None)?;

        // Check that the canvas file is created and contains only the ASK MODEL section
        let canvas_path = project_path.join("legatio.md");
        let canvas_content = fs::read_to_string(&canvas_path)?;

        // Verify that only the placeholder text exists
        assert_eq!(canvas_content.trim(), "# ASK MODEL BELLOW");

        Ok(())
    }
}

