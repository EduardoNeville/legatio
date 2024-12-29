use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, FromRow, Debug)]
pub struct Project {
    pub project_id: String,
    pub project_path: String,
}

impl Project {
    pub fn new(path: &String) -> Project {
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
    pub fn new(scroll_id: &String, content: &String, output: &String, next_prompt_id: &String) -> Prompt {
        Prompt {
            prompt_id: Uuid::new_v4().to_string(),
            scroll_id: scroll_id.to_string(),
            content: content.to_string(),
            output: output.to_string(),
            next_prompt_id: next_prompt_id.to_string()
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
    pub fn new(project_id: &String, init_prompt_id: &String) -> Scroll {
        Scroll {
            scroll_id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            init_prompt_id: init_prompt_id.to_string(),
        }
    }
}
