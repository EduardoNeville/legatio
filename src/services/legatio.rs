use std::{fs::{self, File}, io::Write, path::PathBuf};

use sqlx::{Result, SqlitePool};
use crate::{
    db::{
        project::{delete_project, get_projects, store_project}, prompt::{get_prompts, store_prompt}, scroll::{delete_scroll, get_scrolls, store_scrolls}
    },
    services::{model::get_openai_response, search::select_files, ui::usr_ask},
    utils::{
        file_utils::{get_contents, read_files}, 
        prompt_utils::{prompt_chain, system_prompt},
        structs::{AppState, Project, Prompt, Scroll}
    }
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
            self.state = AppState::SelectProject;
        }

        loop {
            let current_state = &self.state; // Clone the state to avoid borrowing issues
            match current_state {
                AppState::NewProject => {
                    // Handle creating a new project
                    self.state = self.handle_new_project(
                        pool,
                        &project_base_path
                    ).await.unwrap();
                }
                AppState::SelectProject => {
                    self.state = self.handle_select_project(pool).await.unwrap();
                }
                AppState::SelectPrompt => {
                    self.state = self.handle_select_prompt(pool).await.unwrap();
                }
                AppState::AskModel => {
                    self.state = self.handle_ask_model(pool).await.unwrap();
                }
                AppState::EditScrolls => {
                    self.state = self.handle_edit_scrolls(pool).await.unwrap();
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
                    return Ok(AppState::AskModel);
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
                self.current_project = Some(projects[choice].clone());
                let prompts = get_prompts(
                    pool,
                    &self.current_project.as_ref().unwrap().project_id
                ).await.unwrap();

                if prompts.is_empty() {
                    return Ok(AppState::AskModel);
                } else {
                    return Ok(AppState::SelectPrompt);
                }
            } else if choice == projects_len {
                // Create a new project
                return Ok(AppState::NewProject);
            } else if choice == projects_len + 1 {
                // Delete current project
                //delete_project(pool, &self.current_project.unwrap().project_id).await.unwrap();
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
    ) -> Result<AppState> {
        loop {
            let prompts = get_prompts(
                pool,
                &self.current_project.as_ref().unwrap().project_id
            ).await.unwrap();

            if !prompts.is_empty() {
                println!("Select a prompt branch: ");
                usr_prompts(
                    pool,
                    &self.current_project.as_ref().unwrap().project_id
                ).await.unwrap();
            }

            let prompts_len = prompts.len();
            println!("[{}]: Delete Prompt", prompts_len + 1);
            println!("[{}]: Select Project", prompts_len + 2);

            let choice = usr_ask("Enter your choice: " ).unwrap();

            if choice < prompts_len {
                let prompt = prompts.iter()
                    .find(|p| p.idx == choice as i32);
                self.current_prompt = Some(prompt.unwrap().to_owned());

                return Ok(AppState::AskModel);
            } else if choice == prompts_len {
                todo!("Delete prompt");
            } else if choice == prompts_len + 1 {
                return Ok(AppState::SelectProject);
            } else {
                println!("Invalid input, try again.");
            }
        }
    }

    async fn handle_ask_model(
        &self,
        pool: &SqlitePool,
    ) -> Result<AppState> {
        loop {

            let scrolls = get_scrolls(
                pool,
                &self.current_project.as_ref().unwrap().project_id
            ).await.unwrap();
            let prompt = &self.current_prompt;
            let curr_prompt = fs::read_to_string(
                &PathBuf::from(
                    &self.current_project.as_ref().unwrap().project_path
                ).join("legatio.md")
            );

            if !prompt.is_some() && !curr_prompt.is_ok() {
                let mut file = File::create(
                    &PathBuf::from(
                        &self.current_project.as_ref().unwrap().project_path
                    ).join("legatio.md")
                ).expect("Could not create file!");

                for (i,f) in scrolls.iter().enumerate() { 
                    let scroll_name = format!(
                        "[{}] {:?}",
                        i,
                        f.scroll_path.split("/").last()
                    );
                    file.write(&scroll_name.as_bytes()).unwrap();
                }
            }

            // Menu
            println!("Select an option:");
            println!("[{}] Ask the Model", 1);
            println!("[{}] Switch branch", 2);
            println!("[{}] Edit Scrolls", 3);
            println!("[{}] Switch project",4);

            let choice = usr_ask("Enter your choice: \n").unwrap();
            match choice  {
                1 => { 
                    let sys_prompt = system_prompt(&scrolls);
                    let prompts = get_prompts(
                        pool, 
                        &self.current_project.as_ref().unwrap().project_id
                    ).await.unwrap();

                    let mut user_input = String::from("");
                    if self.current_prompt.is_some() {
                        let prompt_chain = prompt_chain(
                            &prompts,
                            &self.current_prompt.as_ref().unwrap()
                        );

                        let _ = prompt_chain.iter().map(|p| {
                            user_input.push_str(&p.content);
                            user_input.push_str(&p.output);
                        });
                    }

                    let curr_prompt = fs::read_to_string(
                        &PathBuf::from(
                            &self.current_project.as_ref().unwrap().project_path
                        ).join("legatio.md")
                    ).unwrap();

                    user_input.push_str(&curr_prompt);

                    let output = get_openai_response(
                        &sys_prompt,
                        &user_input
                    ).await.unwrap();
                    
                    let lst_prompt = Prompt::new(
                        &self.current_project.as_ref().unwrap().project_id,
                        &curr_prompt,
                        &output,
                        if self.current_prompt.is_some() {
                            &self.current_prompt.as_ref().unwrap().prompt_id
                        } else {
                            &self.current_project.as_ref().unwrap().project_id
                        },
                        if self.current_prompt.is_some() {
                            &(self.current_prompt.as_ref().unwrap().idx + 1)
                        } else {
                            &1
                        },
                    );
                    store_prompt(pool, &lst_prompt);

                }, 
                2 => { 
                    return Ok(AppState::SelectPrompt)
                }, 
                3 => { 
                    return Ok(AppState::EditScrolls)
                }, 
                4 => { 
                    return Ok(AppState::SelectProject)
                }, 
                _ => { 
                    println!("Invalid index.");
                }
            }

        }
    }

    async fn handle_edit_scrolls(
        &self,
        pool: &SqlitePool,
    ) -> Result<AppState> {
        loop {
            let scrolls = get_scrolls(
                pool,
                &self.current_project.as_ref().unwrap().project_id
            ).await.unwrap();
            println!("Scrolls in Project:");
            
            // Show all scrolls
            for (idx, scroll) in scrolls.iter().enumerate() {
                println!(" [{}]: {}", idx, scroll.scroll_path);
            }

            // Menu
            println!("Select an option: \n [{}]. Append a Scroll \n [{}]. Remove a Scroll \n [{}]. Select Prompt \n [{}]. Select Project",
                scrolls.len(),
                scrolls.len() + 1,
                scrolls.len() + 2,
                scrolls.len() + 3,
            );

            let choice = usr_ask("Enter your choice: ").unwrap();

            if choice < scrolls.len() {
                println!("Selected scroll: {}", scrolls[choice].scroll_path);
                // Logic to view/edit the scroll
            } else if choice == scrolls.len() {
                self.scroll_append_ctrl(
                    pool,
                    &self.current_project.as_ref().unwrap()
                ).await.unwrap();
            } else if choice == scrolls.len() + 1 {
                // Remove scroll
                let scroll_idx = usr_ask("Select scroll index to remove: ").unwrap();
                if scroll_idx < scrolls.len() {
                    delete_scroll(pool, &scrolls[scroll_idx]).await.unwrap();
                    println!("Removed scroll: {}", scrolls[scroll_idx].scroll_path);
                } else {
                    println!("Invalid index.");
                }
            } else if choice == scrolls.len() + 2 {
                return Ok(AppState::SelectPrompt);
            } else if choice == scrolls.len() + 3 {
                return Ok(AppState::SelectProject);
            } else {
                println!("Invalid input, try again.");
            }
        }
    }


    // Utility Functions
    async fn scroll_append_ctrl(&self, pool: &SqlitePool, project: &Project) -> Result<Vec<Scroll>> {
        let scrolls_in_dir = get_contents(&project.project_path, false, 20).unwrap();
        let selected_scrolls = select_files(scrolls_in_dir, true).unwrap();
        let scrolls = read_files(&selected_scrolls, &project.project_id).unwrap();
        store_scrolls(pool, &scrolls).await.unwrap();
        Ok(get_scrolls(pool, &project.project_id).await.unwrap())
    }

}
