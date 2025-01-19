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

        log_info(&format!(
            "Empty Chain: {}, for prompt: {}",
            chain.is_empty(),
            prompt.prompt_id
        ));

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
    log_error(&format!("The canvas file does not include the '# ASK MODEL BELLOW' marker."));
    Ok(String::new())
}

