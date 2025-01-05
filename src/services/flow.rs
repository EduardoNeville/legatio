use std::io;
use std::io::Write;
use sqlx::{Result, SqlitePool};

use crate::services::model::get_openai_response;
use crate::services::ui::usr_ask;
//use crate::utils::db_utils::highlight_text;
use crate::{
    services::search::select_files, 
    utils::{
        file_utils::{get_contents, read_files}, 
        prompt_utils::construct_system_prompt, 
        structs::{File, Project, Prompt, Scroll}
    }
};
use crate::db::project::{store_project, get_projects};
use crate::db::file::{store_files, get_files};
use crate::db::scroll::{store_scroll, get_scrolls, update_scroll};
use crate::db::prompt::{store_prompt, get_prompts_from_scroll, update_prompt};

pub async fn flow(pool: &SqlitePool)-> Result<()> {
    let home = String::from("/home/eduardoneville/Desktop/");

    loop {
        let projects = get_projects(pool).await.unwrap();
        let mut sel = String::from("Select your project: \n");
        for (idx, project) in projects.iter().enumerate() {
            sel.push_str(
                &format!(" [{:?}]: {:?} \n", 
                    idx, 
                    project.project_path.split("/").last().unwrap()
                )
            );
        }
        sel.push_str(&format!(" [{:?}]: New Project \n", projects.len()));

        let val = usr_ask(&sel).unwrap();
        let mut sel_proj = false;
        let mut sel_scroll = false;
        let project: Project;
        if projects.len() > val {
            let val_proj = projects.get(val).unwrap();

            project = Project {
                project_id: val_proj.project_id.to_string(),
                project_path: val_proj.project_path.to_string(),
            };
            sel_proj = true;

        } else {
            let dir_list = get_contents(&home, true, 5);
            let selected_dir = select_files(dir_list.unwrap(), false).unwrap();

            project = Project::new(&selected_dir[0]);

            let _ = store_project(&pool, &project).await.unwrap();

            sel_proj = true;
        }

        while sel_proj {

            let ans = usr_ask(&String::from(" [0] Append file(s) \n [1] Select scroll \n [2] New Scroll \n [3] Change Project")).unwrap();

            let mut scroll = Scroll::new(&project.project_id, &String::from(""));
            match ans {
                0 => {
                    let files = file_ctrl(pool, &project).await.unwrap();

                    println!("Current files: \n");
                    for (idx, row) in files.iter().enumerate() {
                        println!(" [{}]: {} \n", idx, row.file_path.split("/").last().unwrap());
                    }
                },
                1 => {
                    scroll = scroll_ctrl(pool, &project).await.unwrap();
                    println!("Scroll selected: \n {:?}", scroll);
                    sel_scroll = true;
                },
                2 => {
                    let _ = store_scroll(pool, &scroll).await;
                    let _ = prompt_ctrl(pool, &scroll).await.unwrap();
                    sel_scroll = true;
                },
                3 => { sel_proj = false; },
                _   => println!("Give an answer!"),
            }

            while sel_scroll {

                let ans = usr_ask(&String::from(" [0] Write prompt into scroll \n [1] Ask model \n [2] Change scroll \n [3] Change project")).unwrap();

                match ans {
                    0 => {
                        let _ = prompt_ctrl(pool, &scroll).await.unwrap();
                    },
                    1 => {
                        let _ = ask_ctrl(pool, &project, &scroll).await;
                    },
                    2 => { sel_scroll = false; },
                    3 => { sel_proj = false; sel_scroll = false; }
                    _   => println!("Give an answer!"),
                }
                
            }
        }
    }
}

async fn file_ctrl(pool: &SqlitePool, project: &Project)-> Result<Vec<File>> {
    let files_in_dir = get_contents(&project.project_path, false, 7);
    let selected_files = select_files(files_in_dir.unwrap(), true).unwrap();
    let files = read_files(&selected_files, &project.project_id).unwrap();
    store_files(&pool, &files)
        .await
        .unwrap();

    let files = get_files(pool, &project.project_id).await.unwrap();
    Ok(files)
}

async fn prompt_ctrl(pool: &SqlitePool, scroll: &Scroll)-> Result<Prompt> {
    // Ask the user for a prompt
    print!("Enter your prompt: ");
    io::stdout().flush()?;

    let mut user_prompt = String::new();
    io::stdin().read_line(&mut user_prompt)?;
    let user_prompt = user_prompt.trim().to_string(); // Remove any trailing newline

    let usr_prompt = Prompt::new(
        &scroll.scroll_id,
        &user_prompt,
        &String::new(),
        &String::new()
    );


    let _ = update_scroll(pool, &scroll, &usr_prompt).await;

    // Store the user's prompt
    store_prompt(&pool, &usr_prompt).await.unwrap();
    //let txt = highlight_text("", &usr_prompt.content);
    //println!("Prompt: \n {:?}", txt.unwrap());

    Ok(usr_prompt)
}

async fn ask_ctrl(pool: &SqlitePool, project: &Project, scroll: &Scroll)-> Result<()>{
    let files: Vec<File> = get_files(pool, &project.project_id).await.unwrap();

    let system_prompt = construct_system_prompt(&files).await.unwrap();

    let prompts: Vec<Prompt> = get_prompts_from_scroll(pool, &scroll.scroll_id).await.unwrap();
    let user_prompt = prompts.last().unwrap();

    println!("\n--- Displaying Prompts ---");
    println!("System Prompt:\n{}", system_prompt);
    println!("User Prompt:\n{:?}", user_prompt);
    
    let answer = get_openai_response(&system_prompt, &user_prompt.content).await.unwrap();
    println!("Answer: \n {:?}", answer);

    let _ = update_prompt(pool, &user_prompt, &answer).await.unwrap();

    Ok(())

}

async fn scroll_ctrl(pool: &SqlitePool, project: &Project)-> Result<Scroll>{
    let scrolls = get_scrolls(pool, &project.project_id).await.unwrap();
    
    let mut sel = String::from("Choose a scroll: \n");
    for (idx, scroll) in scrolls.iter().enumerate() {
        sel.push_str(
            &format!(" [{:?}]: {:?} \n", 
                idx, 
                scroll.scroll_id
            )
        );
    }

    sel.push_str(&format!(" [{:?}]: New Scroll \n", scrolls.len()));

    let val = usr_ask(&sel).unwrap();

    let mut scroll = Scroll::new(&project.project_id, &String::from(""));
    if scrolls.len() > val {
        let val_scroll = scrolls.get(val).unwrap();

        scroll = Scroll {
            scroll_id:      val_scroll.scroll_id.to_string(),
            project_id:     val_scroll.project_id.to_string(),
            init_prompt_id: val_scroll.init_prompt_id.to_string(),
        };
    
    } else {
        store_scroll(&pool, &scroll).await.unwrap();
    }
    Ok(scroll)
}

