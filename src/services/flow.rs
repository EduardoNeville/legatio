use std::io;
use std::io::Write;
use sqlx::{Result, SqlitePool};
use crate::{
    db::{ file::{delete_file, get_files, store_files}, project::{delete_project, get_projects, store_project}, prompt::{get_prompts_from_scroll, store_prompt, update_prompt}, scroll::{delete_scroll, get_scrolls, store_scroll}}, services::{model::get_openai_response, ui::usr_ask}, utils::{
        file_utils::{get_contents, read_files},
        prompt_utils::construct_system_prompt,
        structs::{AppState, File, Project, Prompt, Scroll},
    }
};

use super::search::select_files;

pub async fn flow(pool: &SqlitePool) -> Result<()> {
    let home = String::from("/home/eduardoneville/Desktop/");

    // TODO: get_app_state etc...
    let projects = get_projects(pool).await.unwrap();
    let mut state = AppState::NewProject;
    if projects.len() != 0 {
        state = AppState::EditProject(projects[0].to_owned());
    }

    loop {
        match state {
            AppState::NewProject => {
                state = handle_new_project(pool, &home).await?;
            }
            AppState::EditProject(selected_project) => {
                state = handle_edit_project(&pool, &selected_project).await?;
            }
            AppState::NewScroll(selected_project) => {
                state = handle_new_scroll(pool, &selected_project).await?;
            }
            AppState::EditScroll(selected_project, selected_scroll) => {
                state = handle_edit_scroll(&pool, &selected_project, &selected_scroll).await?;
            }
            AppState::EditPrompt(selected_project, selected_scroll) => {
                state = handle_edit_prompts(pool, selected_project, selected_scroll).await?;
            }
            AppState::EditFiles(selected_project, selected_scroll) => {
                state = handle_edit_files(pool, selected_project, selected_scroll).await?;
            }
        }
    }
}

async fn handle_new_project(pool: &SqlitePool, home: &str) -> Result<AppState> {
    loop {
        println!("Welcome! You don't have a Project yet!.\n Options:");
        println!(" [1]: New Project");
        println!(" [2]: Exit");

        let choice = usr_ask("Enter your choice: ").unwrap();

        match choice {
            1 => {
                // Create a new project
                let dir_list = get_contents(home, true, 5).unwrap();
                let selected_dir = select_files(dir_list, false).unwrap();
                let project = Project::new(&selected_dir[0]);
                store_project(pool, &project).await.unwrap();
                println!("New project '{}' created.", project.project_path);
                return Ok(AppState::NewScroll(project)); // NewScroll for the project
            }
            2 => {
                println!("Exiting application...");
                std::process::exit(0);
            }
            _ => {
                println!("Invalid choice, please try again.");
            }
        }
    }
}

async fn handle_edit_project(
    pool: &SqlitePool,
    current_project: &Project,
) -> Result<AppState> {
    loop {
        let projects = get_projects(pool).await.unwrap();
        let mut prompt = String::from("Edit Project: \n");
        //TODO add colour to check which project is the current project
        for (idx, project) in projects.iter().enumerate() {
            prompt.push_str(&format!(
                " [{}]: {:?} \n", 
                idx, 
                project.project_path.split("/").last().unwrap()
            ));
        }
        
        let mut curr_idx = projects.len();
        prompt.push_str(&format!(" [{}]: New Project \n", curr_idx));
        curr_idx += 1;

        prompt.push_str(&format!(" [{}]: Delete Current Project \n", curr_idx));
        curr_idx += 1;

        prompt.push_str(&format!(" [{}]: Exit \n", curr_idx));

        let val = usr_ask(&prompt).unwrap();

        if val < projects.len() {
            // Select existing project
            let selected_project = projects[val].to_owned();
            let scrolls = get_scrolls(pool, &selected_project.project_id).await.unwrap();
            let mut state = AppState::NewScroll(selected_project.to_owned());
            if scrolls.len() != 0 {
                state = AppState::EditScroll(selected_project.to_owned(), scrolls[0].to_owned());
            }
            return Ok(state);
        } else if val == projects.len() {
            // Create new project
            return Ok(AppState::NewProject);
        } else if val == projects.len() + 1 {
            // Delete current project
            delete_project(pool, &current_project).await.unwrap();
            if projects.len() == 1 { // Last project deleted
                return Ok(AppState::NewProject); // Transition back to None state
            }
            println!("Project deleted.");    
        } else if val == projects.len() + 2 {
            // Exit
            println!("Exiting application...");
            std::process::exit(0);
        } else {
            println!("Invalid input, try again.");
        }
    }
}

async fn handle_new_scroll(pool: &SqlitePool, project: &Project) -> Result<AppState> {
    loop {
        println!("Creating a new Scroll.\nOptions:");
        println!(" [1]: New Scroll");
        println!(" [2]: Exit");

        let choice = usr_ask("Enter your choice: ").unwrap();

        match choice {
            1 => {
                // Create a new project
                let new_scroll = Scroll::new(&project.project_id, "");
                store_scroll(pool, &new_scroll).await.unwrap();
                AppState::EditPrompt(project.to_owned(), new_scroll.to_owned());
                println!("New scroll created.");
            }
            2 => {
                println!("Exiting application...");
                std::process::exit(0);
            }
            _ => {
                println!("Invalid choice, please try again.");
            }
        }
    }
}
async fn handle_edit_scroll(
    pool: &SqlitePool, 
    project: &Project, 
    current_scroll: &Scroll
) -> Result<AppState> {
    loop {
        let scrolls = get_scrolls(pool, &project.project_id).await.unwrap();
        let mut prompt = String::from("Edit Scroll: \n");

        for (idx, scroll) in scrolls.iter().enumerate() {
            prompt.push_str(&format!(" [{}]: {:?} \n", idx, scroll.scroll_id));
        }

        let mut curr_idx = scrolls.len();
        prompt.push_str(&format!(" [{}]: New Scroll \n", curr_idx));
        curr_idx += 1;

        if &current_scroll.init_prompt_id != "" {
            prompt.push_str(&format!(" [{}]: Delete Current Scroll \n", curr_idx));
            curr_idx += 1;
        }
        // len + 1
        prompt.push_str(&format!(" [{}]: Edit Files \n", curr_idx));
        curr_idx += 1;
        // len + 2
        prompt.push_str(&format!(" [{}]: Back to Edit Projects \n", curr_idx));

        let val = usr_ask(&prompt).unwrap();

        if val < scrolls.len() {
            // Select an existing scroll
            let selected_scroll = scrolls[val].clone();
            println!("Scroll selected. Proceeding to edit.");
            return Ok(AppState::EditPrompt(project.to_owned(), selected_scroll.to_owned()));
        } else if val == scrolls.len() {
            // Create a new scroll
            let new_scroll = Scroll::new(&project.project_id, "");
            store_scroll(pool, &new_scroll).await.unwrap();
            println!("New scroll created.");
        } else if val == scrolls.len() + 1 {
            // Delete current scroll
            delete_scroll(pool, &current_scroll).await.unwrap();

            if scrolls.len() == 1 { // Last project deleted
                return Ok(AppState::NewScroll(project.to_owned())); // Transition back to None state
            }
            println!("Scroll deleted.");
        } else if val == scrolls.len() + 2 {
            // Edit Files
            return Ok(AppState::EditFiles(project.to_owned(), current_scroll.to_owned()));
        } else if val == scrolls.len() + 3 {
            // Back to project editing
            return Ok(AppState::EditProject(project.to_owned()));
        } else {
            println!("Invalid input, try again.");
        }
    }
}

async fn handle_edit_prompts(
    pool: &SqlitePool,
    project: Project,
    scroll: Scroll,
) -> Result<AppState> {
    loop {
        let prompts = get_prompts_from_scroll(pool, &scroll.scroll_id).await.unwrap();
        println!("Prompts in Scroll: {}", &scroll.scroll_id );
        
        // Display all prompts in order
        for (idx, prompt) in prompts.iter().enumerate() {
            println!(" [{}]: {}", idx, prompt.content);
        }

        // Menu
        println!(
            "Select an option:
            [1]. Generate a .md file
            [2]. Use a specific prompt
            [3]. Change prompt branch
            [4]. Edit Scroll"
        );
        let choice = usr_ask("Enter your choice: ").unwrap();

        match choice {
            1 => {
                println!("Generating .md file...");
                // Logic for .md generation goes here
            }
            2 => {
                let prompt_idx = usr_ask("Select prompt index: ").unwrap();
                if prompt_idx < prompts.len() {
                    println!("Using Prompt [{}]: {}", prompt_idx, prompts[prompt_idx].content);
                    // Logic to use the specific prompt
                } else {
                    println!("Invalid index.");
                }
            }
            3 => {
                let prompt_idx = usr_ask("Select branch index to edit: ").unwrap();
                if prompt_idx < prompts.len() {
                    // Logic for changing a prompt branch
                    println!("Editing prompt branch...");
                } else {
                    println!("Invalid index.");
                }
            }
            4 => return Ok(AppState::EditScroll(project, scroll)),
            _ => println!("Invalid input. Try again."),
        }
    }
}

async fn handle_edit_files(
    pool: &SqlitePool,
    project: Project,
    scroll: Scroll,
) -> Result<AppState> {
    loop {
        let files = get_files(pool, &project.project_id).await.unwrap();
        println!("Files in Project:");
        
        // Show all files
        for (idx, file) in files.iter().enumerate() {
            println!(" [{}]: {}", idx, file.file_path);
        }

        // Menu
        println!("Select an option: \n [{}]. Append a File \n [{}]. Remove a File \n [{}]. Edit Prompt \n [{}]. Edit Scroll \n [{}]. Edit Project",
            files.len(),
            files.len() + 1,
            files.len() + 2,
            files.len() + 3,
            files.len() + 4
        );

        let choice = usr_ask("Enter your choice: ").unwrap();

        if choice < files.len() {
            println!("Selected file: {}", files[choice].file_path);
            // Logic to view/edit the file
        } else if choice == files.len() {
            file_append_ctrl(pool, &project).await.unwrap();
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
            return Ok(AppState::EditPrompt(project.clone(), scroll.clone()));
        } else if choice == files.len() + 3 {
            return Ok(AppState::EditScroll(project, scroll));
        } else if choice == files.len() + 4 {
            return Ok(AppState::EditProject(project));
        } else {
            println!("Invalid input, try again.");
        }
    }
}


// Utility Functions

async fn file_append_ctrl(pool: &SqlitePool, project: &Project) -> Result<Vec<File>> {
    let files_in_dir = get_contents(&project.project_path, false, 20).unwrap();
    let selected_files = select_files(files_in_dir, true).unwrap();
    let files = read_files(&selected_files, &project.project_id).unwrap();
    store_files(pool, &files).await.unwrap();
    Ok(get_files(pool, &project.project_id).await.unwrap())
}

async fn prompt_ctrl(pool: &SqlitePool, scroll: &Scroll) -> Result<Prompt> {
    print!("Enter your prompt: ");
    io::stdout().flush()?;
    let mut user_prompt = String::new();
    io::stdin().read_line(&mut user_prompt)?;
    let user_prompt = user_prompt.trim().to_string();

    let prompt = Prompt::new(&scroll.scroll_id, &user_prompt, "", "");
    store_prompt(pool, &prompt).await.unwrap();
    Ok(prompt)
}
