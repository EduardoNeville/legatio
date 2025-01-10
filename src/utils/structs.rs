use sqlx::FromRow;
use uuid::Uuid;

pub enum AppState {
    NewProject,
    EditProject(Project),
    NewScroll(Project),
    EditScroll(Project, Scroll),
    EditPrompt(Project, Scroll),
    EditFiles(Project, Scroll),
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
    pub scroll_id: String,
    pub content: String,
    pub output: String,
    pub next_prompt_id: String,
}

impl Prompt {
    pub fn new(scroll_id: &str, content: &str, output: &str, next_prompt_id: &str) -> Prompt {
        Prompt {
            prompt_id: Uuid::new_v4().to_string(),
            scroll_id: scroll_id.to_string(),
            content: content.to_string(),
            output: output.to_string(),
            next_prompt_id: next_prompt_id.to_string(),
        }
    }
}

#[derive(Clone, FromRow, Debug)]
pub struct Scroll {
    pub scroll_id: String,
    pub project_id: String,
    pub init_prompt_id: String,
}

impl Scroll {
    pub fn new(project_id: &str, init_prompt_id: &str) -> Scroll {
        Scroll {
            scroll_id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            init_prompt_id: init_prompt_id.to_string(),
        }
    }
}
