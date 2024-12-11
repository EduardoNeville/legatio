use uuid::Uuid;

#[derive(Debug)]
pub struct Project {
    pub project_id: String,
    pub project_path: String,
    pub files: String,
}

impl Project {
    pub fn new(path: &String, files: &String) -> Project {
        Project {
            project_id: Uuid::new_v4().to_string(),
            project_path: path.to_string(),
            files: files.to_string(),
        }
    }
}


#[derive(Debug)]
pub struct File {
    pub file_id: String,
    pub file_path: String,
    pub content: String,
}

impl File {
    pub fn new(path: &String, content: &String) -> File {
        File {
            file_id: Uuid::new_v4().to_string(),
            file_path: path.to_string(),
            content: content.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Prompt {
    pub prompt_id: String,
    pub content: String,
    pub output: String,
}

impl Prompt {
    pub fn new(prompt: &String, output: &String) -> Prompt {
        Prompt {
            prompt_id: Uuid::new_v4().to_string(),
            content: prompt.to_string(),
            output: output.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Scrolls {
    pub scroll_id: String,
    pub project_id: String,
    pub prompts: String,
}

impl Scrolls {
    pub fn new(project_id: &String, prompts: &String) -> Scrolls {
        Scrolls {
            scroll_id: Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            prompts: prompts.to_string(),
        }
    }
}
