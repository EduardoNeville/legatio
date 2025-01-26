use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use anyhow::{Result, Context};
use crate::utils::logger::{log_error, log_info};
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

    // Open the file for writing, truncating its content to clear everything initially
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true) // Clear the file content
        .create(true)   // Create the file if it doesn't exist
        .open(&file_path)
        .with_context(|| format!("Failed to open canvas file {:?}", file_path))?;

    // Proceed only if both a list of prompts and a single prompt are provided
    if let (Some(prompts), Some(prompt)) = (prompts, prompt) {
        // Generate the prompt chain
        let mut chain = prompt_chain(prompts, prompt);
        chain.reverse(); // Reverse the chain order if needed

        if chain.is_empty() {
            log_info("No chain generated");
        }

        // Write each prompt and its output to the file manually (without writeln)
        for prompt in chain {
            let prompt_text = format!("# PROMPT {}\n{}\n", prompt.prompt_id, prompt.content);
            let output_text = format!("# OUTPUT {}\n{}\n", prompt.prompt_id, prompt.output);

            // Write prompt text
            if let Err(err) = file.write_all(prompt_text.as_bytes()) {
                log_error(&format!(
                    "Failed to write prompt to canvas file: {:?}",
                    err
                ));
                return Err(err.into());
            }

            // Write output text
            if let Err(err) = file.write_all(output_text.as_bytes()) {
                log_error(&format!(
                    "Failed to write output to canvas file: {:?}",
                    err
                ));
                return Err(err.into());
            }
        }
    }

    // Add the placeholder section for asking the model below (without writeln)
    let placeholder = "\n# ASK MODEL BELLOW";
    if let Err(err) = file.write_all(placeholder.as_bytes()) {
        log_error(&format!(
            "Failed to write ASK MODEL section to canvas file: {:?}",
            err
        ));
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
    log_error("The canvas file does not include the '# ASK MODEL BELLOW' marker.");
    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir; // Import tempfile for temporary directories
    use std::fs::{self, File};
    use std::io::Write;
    use crate::utils::structs::{Project, Prompt};
    use anyhow::Result;

    #[test]
    fn test_chain_into_canvas_creates_file() -> Result<()> {
        // Arrange: Create a temporary directory and mock project
        let temp_dir = tempdir()?; // Create temporary directory
        let project_path = temp_dir.path().to_str().unwrap().to_string();
        let project = Project {
            project_id: "test_project".to_string(),
            project_path: project_path.clone(),
        };

        let prompts = vec![
            Prompt {
                prompt_id: "prompt1".to_string(),
                project_id: project.project_id.clone(),
                prev_prompt_id: "root".to_string(),
                content: "Test Prompt 1".to_string(),
                output: "Output 1".to_string(),
            },
        ];
        let prompt = &prompts[0];

        // Act: Call the target function
        chain_into_canvas(&project, Some(&prompts), Some(prompt))?;

        // Assert: Verify `legatio.md` is created with the correct content
        let canvas_path = temp_dir.path().join("legatio.md");
        let content = fs::read_to_string(&canvas_path)?;
        assert!(content.contains("Test Prompt 1"));
        assert!(content.contains("Output 1"));
        assert!(content.contains("# ASK MODEL BELLOW"));

        Ok(())
    }



    #[test]
    fn test_chain_match_canvas_finds_marker() -> Result<()> {
        // Arrange: Create a temporary directory and mock project
        let temp_dir = tempdir()?;
        let project_path = temp_dir.path().to_str().unwrap().to_string();
        let project = Project {
            project_id: "test_project".to_string(),
            project_path: project_path.clone(),
        };

        // Create the `legatio.md` with mock content
        let canvas_path = temp_dir.path().join("legatio.md");
        {
            let mut file = File::create(&canvas_path)?;
            file.write_all(b"# PROMPT ID 1\nPrompt Content\n# OUTPUT ID 1\nOutput Content\n# ASK MODEL BELLOW\nExtra User Input")?;
        }

        // Act: Call the target function to check for unmatched content
        let unmatched_content = chain_match_canvas(&project)?;

        // Assert: Verify everything after the marker is returned
        assert_eq!(unmatched_content, "\nExtra User Input");

        Ok(())
    }

    #[test]
    fn test_chain_match_canvas_handles_no_marker() -> Result<()> {
        // Arrange: Create a temporary directory and mock project
        let temp_dir = tempdir()?;
        let project_path = temp_dir.path().to_str().unwrap().to_string();
        let project = Project {
            project_id: "test_project".to_string(),
            project_path: project_path.clone(),
        };

        // Create the `legatio.md` without the marker
        let canvas_path = temp_dir.path().join("legatio.md");
        {
            let mut file = File::create(&canvas_path)?;
            file.write_all(b"# PROMPT ID 1\nNo marker here")?;
        }

        // Act: Call the target function
        let unmatched_content = chain_match_canvas(&project)?;

        // Assert: Verify unmatched content is empty when marker is not found
        assert_eq!(unmatched_content, "");

        Ok(())
    }

    #[test]
    fn test_chain_into_canvas_with_no_prompts_does_not_fail() -> Result<()> {
        // Arrange: Create a temporary directory and mock project
        let temp_dir = tempdir()?;
        let project_path = temp_dir.path().to_str().unwrap().to_string();
        let project = Project {
            project_id: "test_project".to_string(),
            project_path: project_path.clone(),
        };

        // Act: Call the target function with empty inputs
        let result = chain_into_canvas(&project, None, None);

        // Assert: Ensure the function succeeds
        assert!(result.is_ok());

        // Verify that the file contains only the placeholder
        let canvas_path = temp_dir.path().join("legatio.md");
        let content = fs::read_to_string(&canvas_path)?;
        assert!(content.contains("# ASK MODEL BELLOW"));

        Ok(())
    }
}

