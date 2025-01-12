use sqlx::FromRow;
use uuid::Uuid;

pub enum AppState {
    NewProject,
    SelectProject(Project),
    SelectPrompt(Project),
    AskModel(Project, Prompt),
    EditFiles(Project),
}

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

#[derive(Clone, FromRow, Debug)]
pub struct File {
    pub file_id: String,
    pub file_path: String,
    pub content: String,
    pub project_id: String,
}

impl File {
    pub fn new(path: &String, content: &String, project_id: &String) -> File {
        File {
            file_id: Uuid::new_v4().to_string(),
            file_path: path.to_string(),
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
    pub idx: i32,
}

impl Prompt {
    pub fn new(project_id: &str, content: &str, output: &str, prev_prompt_id: &str, idx: &i32) -> Prompt {
        Prompt {
            prompt_id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            content: content.to_string(),
            output: output.to_string(),
            prev_prompt_id: prev_prompt_id.to_string(),
            idx: *idx,
        }
    }
}

