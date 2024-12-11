use dotenv::dotenv;
use utils::structs::{File, Project};
use std::env;

mod services;
mod utils;

use anyhow::Result;

use utils::db_utils::{ get_db_pool, store_project, store_files};
use utils::file_utils::{ get_contents, read_files };
use utils::prompt_utils::construct_system_prompt;
use services::model::get_openai_response;
use services::search::select_files;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let api_key = env::var("OPENAI_API_KEY")?;

    let pool = get_db_pool().await?;

    let home = String::from("/home/eduardoneville/Desktop/AiRs");

    // Get all directories
    let dir_list = get_contents(home, true);
    let selected_dir = select_files(dir_list.unwrap())?;
    let project_dir = selected_dir[0].clone();
    println!("Directory Selected: {:?}", selected_dir);

    store_project(&pool, &Project::new(&project_dir, &"".to_string()));

    let files_in_dir = get_contents(project_dir.clone(), false);
    let selected_files = select_files(files_in_dir.unwrap())?;

    println!("Files Selected: \n {:?}, ", selected_files);

    let files = read_files(&selected_files)?;

    //store_files(&pool, &project_dir.clone(), &files).await?;

    //let system_prompt = construct_system_prompt(&files);

    //// Display the system prompt
    //println!("System prompt:\n{}", system_prompt);

    //// Ask the user for a prompt
    //print!("Enter your prompt: ");
    //io::stdout().flush()?; // Ensure the prompt is displayed immediately

    //let mut user_prompt = String::new();
    //io::stdin().read_line(&mut user_prompt)?;
    //let user_prompt = user_prompt.trim().to_string(); // Remove any trailing newline

    //// Store the user's prompt
    //store_prompt(&pool, &user_prompt).await?;

    //// Display both prompts
    //println!("\n--- Displaying Prompts ---");
    //println!("System Prompt:\n{}", system_prompt);
    //println!("User Prompt:\n{}", user_prompt);
    //
    //let ans = get_openai_response(&api_key, &system_prompt, &user_prompt).await?;

    //println!("GPT Answer:\n{}", ans);

    Ok(())
}
