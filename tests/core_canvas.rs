#[cfg(test)]
mod tests {
    use anyhow::Result;
    use legatio::{
        core::canvas::*,
        utils::structs::{Project, Prompt},
    };
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir; // Import tempfile for temporary directories

    #[test]
    fn test_chain_into_canvas_creates_file() -> Result<()> {
        // Arrange: Create a temporary directory and mock project
        let temp_dir = tempdir()?; // Create temporary directory
        let project_path = temp_dir.path().to_str().unwrap().to_string();
        let project = Project {
            project_id: "test_project".to_string(),
            project_path: project_path.clone(),
        };

        let prompts = vec![Prompt {
            prompt_id: "prompt1".to_string(),
            project_id: project.project_id.clone(),
            prev_prompt_id: "root".to_string(),
            content: "Test Prompt 1".to_string(),
            output: "Output 1".to_string(),
        }];
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
