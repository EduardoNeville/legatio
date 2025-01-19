use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use anyhow::{Result, Context};
use crate::{
    core::prompt::prompt_chain,
    utils::structs::{Prompt, Project}
};

pub fn chain_into_canvas(project: &Project, prompts: &[Prompt], prompt: &Prompt)-> Result<()> {
    let mut chain = prompt_chain(prompts, prompt);
    chain.reverse();

    let mut file = OpenOptions::new()
        .write(true)
        .open(&PathBuf::from(&project.project_path).join("legatio.md"))
        .unwrap();

    for prompt in chain {
        let content = format!("# PROMPT {} \n{}", prompt.prompt_id, prompt.content);
        writeln!(file, "{}", content).unwrap();

        let content = format!("# OUTPUT {} \n{}", prompt.prompt_id, prompt.output);
        writeln!(file, "{}", content).unwrap();
    }

    Ok(())
}

/// Checks if the sequence of prompts exists in order within the canvas file (`legatio.md`).
///
/// If a prompt within the chain is missing, the function assumes the rest of the file
/// contains new and untracked content, halts further comparison, and returns accordingly.
///
/// # Parameters
/// - `project`: The `Project` object representing the current project.
/// - `prompts`: A slice of all existing `Prompt` objects in the project.
/// - `prompt`: A reference to the `Prompt` object at the head of the chain.
///
/// # Returns
/// - `Ok(())` if all prompts in the chain are found in order within the canvas.
/// - `Err(anyhow::Error)` if the canvas cannot be read or if the chain is disrupted,
///    providing an error context about where the mismatch occurred.
///
/// # Example
/// ```rust
/// let result = chain_match_canvas(project, &prompts, &current_prompt);
/// if let Err(err) = result {
///     eprintln!("Mismatch in prompt chain: {}", err);
/// }
/// ```
pub fn chain_match_canvas(project: &Project, prompts: &[Prompt], prompt: &Prompt) -> Result<String> {
    // 1. Generate the full chain of prompts starting from `prompt`.
    let mut chain = prompt_chain(prompts, prompt);
    chain.push(prompt.to_owned());

    // 2. Read the canvas file (`legatio.md`) from the project's path.
    let canvas_path = PathBuf::from(&project.project_path).join("legatio.md");
    let canvas = fs::read_to_string(&canvas_path)
        .with_context(|| format!("Failed to read canvas file at {:?}", canvas_path))?;

    // 3. Track the cursor position in the canvas to ensure sequential matching.
    let mut search_start_index = 0;

    // 4. Iterate through the chain and verify it in the canvas file sequentially.
    for (index, i_prompt) in chain.iter().enumerate() {
        // Check if the current i_prompt appears in order within the canvas.
        if let Some(match_index) = canvas[search_start_index..].find(&i_prompt.content) {
            // If found, update the cursor to after this match.
            search_start_index += match_index + i_prompt.content.len();
        } else {
            // If a i_prompt in the chain is missing:
            // Assume the rest of the canvas contains new, untracked content.
            let unmatched_content_start = canvas[search_start_index..].to_string();
            return Ok(unmatched_content_start);
        }
    }

    // 5. If the entire chain is matched successfully, return `Ok`.
    Ok(String::new())
}
