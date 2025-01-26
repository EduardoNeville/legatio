use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, FromRow, Debug)]
pub struct Project {
    pub project_id: String,
    pub project_path: String,
}

impl Project {
    pub fn new(path: &str) -> Project {
        Project {
            project_id: Uuid::new_v4().to_string(),
            project_path: path.to_string(),
        }
    }
}

/// Struct representing a Scroll.
///
/// A `Scroll` is associated with a project, and it stores a unique ID, its file path,
/// the file's content, and the project ID it belongs to.
#[derive(Debug, FromRow)]
pub struct Scroll {
    pub scroll_id: String,   // Unique identifier for the Scroll
    pub scroll_path: String, // The file path associated with the Scroll
    pub content: String,     // The content of the Scroll (from the file)
    pub project_id: String,  // The ID of the project the Scroll is associated with
}

impl Scroll {
    pub fn new(path: &str, content: &str, project_id: &str) -> Scroll {
        Scroll {
            scroll_id: Uuid::new_v4().to_string(),
            scroll_path: path.to_string(),
            content: content.to_string(),
            project_id: project_id.to_string(),
        }
    }
}

#[derive(Clone, FromRow, Debug)]
pub struct Prompt {
    pub prompt_id: String,
    pub project_id: String,
    pub content: String,
    pub output: String,
    pub prev_prompt_id: String,
}

impl Prompt {
    pub fn new(project_id: &str, content: &str, output: &str, prev_prompt_id: &str) -> Prompt {
        Prompt {
            prompt_id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            content: content.to_string(),
            output: output.to_string(),
            prev_prompt_id: prev_prompt_id.to_string(),
        }
    }
}
