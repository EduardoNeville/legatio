use sqlx::{Result, SqlitePool};
use crate::{
    db::{file::{delete_file, get_files, store_files}, project::{delete_project, get_projects, store_project}, prompt::{get_prompts, store_prompt}}, services::{search::select_files, ui::usr_ask}, utils::{file_utils::{get_contents, read_files}, structs::{AppState, File, Project, Prompt}}
};

use super::ui::usr_prompts;

pub struct Legatio {
    state: AppState,
    current_project: Option<Project>,
    current_prompt: Option<Prompt>,
}

impl Legatio {
    pub fn new() -> Self {
        Legatio {
            state: AppState::NewProject,
            current_project: None,
            current_prompt: None,
        }
    }

    pub async fn run(&mut self, pool: &SqlitePool) -> Result<()> {
        let project_base_path = String::from("/home/eduardoneville/Desktop/");

        // Fetch projects for initialization
        let projects = get_projects(pool).await.unwrap();
        if !projects.is_empty() {
            self.current_project = Some(projects[0].clone());
            self.state = AppState::SelectProject(self.current_project.clone().unwrap());
        }

        loop {
            let current_state = &self.state; // Clone the state to avoid borrowing issues
            match current_state {
                AppState::NewProject => {
                    // Handle creating a new project
                    self.state = self.handle_new_project(pool, &project_base_path).await.unwrap();
                }
                AppState::SelectProject(project) => {
                    self.state = self.handle_select_project(pool, &project).await.unwrap();
                }
                AppState::SelectPrompt(project) => {
                    self.state = self.handle_select_prompt(pool, &project).await.unwrap();
                }
                AppState::AskModel(project, prompt) => {
                    self.state = self.handle_ask_model(pool, &project, &prompt).await.unwrap();
                }
                AppState::EditFiles(project) => {
                    self.state = self.handle_edit_files(pool, &project).await.unwrap();
                }
            }
        }
    }

    async fn handle_new_project(
        &mut self,
        pool: &SqlitePool,
        base_path: &str,
    ) -> Result<AppState> {
        println!("Welcome! You do not have a project yet.");

        loop {
            println!("Options:\n[1]: Create a new project\n[2]: Exit");

            let choice = usr_ask("Enter your choice: ").unwrap();

            match choice {
                1 => {
                    // Logic to create a new project
                    println!("Creating a new project...");
                    let dir_list = get_contents(base_path, true, 20)
                        .expect("Failed to fetch directory");
                    let selected_dir = select_files(dir_list, false)
                        .expect("Failed to select directory");
                    let project = Project::new(&selected_dir[0]);
                    store_project(pool, &project).await.unwrap();
                    println!("Project '{}' created.", project.project_path);
                    self.current_project = Some(project.clone());
                    return Ok(AppState::AskModel(project, Prompt::new("", "", "", "", &0)));
                }
                2 => {
                    println!("Exiting the application...");
                    std::process::exit(0);
                }
                _ => {
                    println!("Invalid choice, please try again.");
                }
            }
        }
    }

    async fn handle_select_project(
        &mut self,
        pool: &SqlitePool,
        current_project: &Project,
    ) -> Result<AppState> {
        loop {
            let projects = get_projects(pool).await.unwrap();
            println!("Edit Project:");

            for (idx, project) in projects.iter().enumerate() {
                println!(
                    " [{}]: {:?}",
                    idx,
                    project.project_path.split('/').last().unwrap()
                );
            }

            let projects_len = projects.len();
            println!("[{}]: New Project", projects_len);
            println!("[{}]: Delete Current Project", projects_len + 1);
            println!("[{}]: Exit", projects_len + 2);

            let choice = usr_ask("Enter your choice: ").unwrap();

            if choice < projects_len {
                let selected_project = projects[choice].clone();
                let prompts = get_prompts(pool, &selected_project.project_id).await.unwrap();
                if prompts.is_empty() {
                    self.current_project = Some(selected_project.clone());
                    return Ok(AppState::AskModel(selected_project, Prompt::new("", "", "", "", &0)));
                } else {
                    return Ok(AppState::SelectPrompt(
                        selected_project.to_owned(),
                    ));
                }
            } else if choice == projects_len {
                // Create a new project
                return Ok(AppState::NewProject);
            } else if choice == projects_len + 1 {
                // Delete current project
                delete_project(pool, &current_project.project_id).await.unwrap();
                return Ok(AppState::NewProject);
            } else if choice == projects_len + 2 {
                // Exit
                println!("Exiting application...");
                std::process::exit(0);
            } else {
                println!("Invalid input, try again.");
            }
        }
    }

    async fn handle_select_prompt(
        &mut self,
        pool: &SqlitePool,
        project: &Project,
    ) -> Result<AppState> {
        loop {
            let prompts = get_prompts(pool, &project.project_id).await.unwrap();

            if !prompts.is_empty() {
                println!("Select a prompt branch: ");
                usr_prompts(pool, &project.project_id).await.unwrap();
            }

            let prompts_len = prompts.len();
            println!("[{}]: Delete Prompt", prompts_len + 1);
            println!("[{}]: Select Project", prompts_len + 2);

            let choice = usr_ask("Enter your choice: " ).unwrap();

            if choice < prompts_len {
                let prompt = prompts.iter()
                    .find(|p| p.idx == choice as i32).unwrap();
                return Ok(AppState::AskModel(project.clone(), prompt.clone()));
            } else if choice == prompts_len {
                todo!("Delete prompt");
            } else if choice == prompts_len + 1 {
                return Ok(AppState::SelectProject(project.clone()));
            } else {
                println!("Invalid input, try again.");
            }
        }
    }

    async fn handle_ask_model(
        &mut self,
        pool: &SqlitePool,
        project: &Project,
        prompt: &Prompt
    ) -> Result<AppState> {

    }

    async fn handle_edit_files(
        &mut self,
        pool: &SqlitePool,
        project: &Project,
    ) -> Result<AppState> {
        loop {
            let files = get_files(pool, &project.project_id).await.unwrap();
            println!("Files in Project:");
            
            // Show all files
            for (idx, file) in files.iter().enumerate() {
                println!(" [{}]: {}", idx, file.file_path);
            }

            // Menu
            println!("Select an option: \n [{}]. Append a File \n [{}]. Remove a File \n [{}]. Select Prompt \n [{}]. Select Project",
                files.len(),
                files.len() + 1,
                files.len() + 2,
                files.len() + 3,
            );

            let choice = usr_ask("Enter your choice: ").unwrap();

            if choice < files.len() {
                println!("Selected file: {}", files[choice].file_path);
                // Logic to view/edit the file
            } else if choice == files.len() {
                self.file_append_ctrl(pool, &project).await.unwrap();
            } else if choice == files.len() + 1 {
                // Remove file
                let file_idx = usr_ask("Select file index to remove: ").unwrap();
                if file_idx < files.len() {
                    delete_file(pool, &files[file_idx]).await.unwrap();
                    println!("Removed file: {}", files[file_idx].file_path);
                } else {
                    println!("Invalid index.");
                }
            } else if choice == files.len() + 2 {
                return Ok(AppState::SelectPrompt(project.clone()));
            } else if choice == files.len() + 3 {
                return Ok(AppState::SelectProject(project.clone()));
            } else {
                println!("Invalid input, try again.");
            }
        }
    }


    // Utility Functions
    async fn file_append_ctrl(&mut self, pool: &SqlitePool, project: &Project) -> Result<Vec<File>> {
        let files_in_dir = get_contents(&project.project_path, false, 20).unwrap();
        let selected_files = select_files(files_in_dir, true).unwrap();
        let files = read_files(&selected_files, &project.project_id).unwrap();
        store_files(pool, &files).await.unwrap();
        Ok(get_files(pool, &project.project_id).await.unwrap())
    }
}
