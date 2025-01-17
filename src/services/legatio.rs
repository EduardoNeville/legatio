use ratatui::{backend::CrosstermBackend, Terminal};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::style::{Style, Color};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};

use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf
};

use std::io;
use std::io::prelude::*;

use sqlx::{Result, SqlitePool};
use crate::{
    db::{
        project::{delete_project, get_projects, store_project},
        prompt::{delete_prompt, get_prompts, store_prompt},
        scroll::{delete_scroll, get_scrolls, store_scroll}
    },
    services::{
        model::get_openai_response, 
        search::{item_selector, select_files}, 
        ui::{clear_screen, usr_ask, usr_prompt_chain, usr_scrolls},
        display::AppState
    },
    utils::{
        file_utils::read_file, logger::log_info, prompt_utils::{format_prompt, prompt_chain, system_prompt}, structs::{Project, Prompt, Scroll}
    }
};

use super::display::{build_project_list, display_project_list, format_project_title, InputEvent};
use super::ui::usr_prompts;

pub struct Legatio {
    state: AppState,
    current_project: Option<Project>,
    current_prompt: Option<Prompt>,
}

impl Legatio {
    pub fn new() -> Self {
        Legatio {
            state: AppState::SelectProject,
            current_project: None,
            current_prompt: None,
        }
    }

    pub async fn run(&mut self, pool: &SqlitePool) -> Result<()> {
        clear_screen();
        // Initialize terminal
        enable_raw_mode().unwrap();
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        // Fetch projects for initialization
        let projects = get_projects(pool).await.unwrap();
        if !projects.is_empty() {
            self.current_project = Some(projects[0].clone());
            self.state = AppState::SelectProject;
        }

        loop {
            let current_state = &self.state; // Clone the state to avoid borrowing issues
            match current_state {
                AppState::SelectProject => {
                    self.state = self.handle_select_project(
                        &mut terminal,
                        pool
                    ).await.unwrap();
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

    async fn handle_select_project(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        pool: &SqlitePool,
    ) -> Result<AppState> {
        loop {
            clear_screen();

            // Fetch and display projects
            let projects = get_projects(pool).await.unwrap();
            let title = format_project_title(&self.current_project);

            let (items, proj_items) = build_project_list(&projects);
            display_project_list(terminal, &title, items).unwrap();

            // Handle input
            if let Event::Key(key_event) = event::read().unwrap() {
                match InputEvent::from(key_event) {
                    InputEvent::Select => {
                        if !proj_items.is_empty() {
                            let sel_p: String = item_selector(proj_items.clone()).unwrap().unwrap_or_default();
                            let sel_idx = proj_items.iter().position(|p| *p == sel_p).unwrap();
                            self.current_project = Some(projects[sel_idx].clone());
                            return Ok(AppState::SelectPrompt);
                        }
                    }
                    InputEvent::New => {
                        let selected_dir = select_files(None).unwrap();
                        let project = Project::new(&selected_dir);
                        store_project(pool, &project).await.unwrap();
                        self.current_project = Some(project.clone());
                        return Ok(AppState::EditScrolls);
                    }
                    InputEvent::Delete => {
                        if !proj_items.is_empty() {
                            let sel_p: String = item_selector(proj_items.clone()).unwrap().unwrap_or_default();
                            let sel_idx = proj_items.iter().position(|p| *p == sel_p).unwrap();
                            delete_project(pool, &projects[sel_idx].project_id).await.unwrap();
                            continue;
                        }
                    }
                    InputEvent::Quit => break,
                    _ => continue,
                }
            }
        }
        Ok(AppState::SelectProject)
    }

    async fn handle_select_prompt(
        &mut self,
        pool: &SqlitePool,
    ) -> Result<AppState> {
        loop {
            clear_screen();
            // Show prompts
            let prompts = get_prompts(
                pool,
                &self.current_project.as_ref().unwrap().project_id
            ).await.unwrap();

            let proj_prompt = format!(
                "  -[ {} : Unchained ]-",
                &self.current_project.as_ref().unwrap()
                .project_path.split("/").last().unwrap()
            );

            if !prompts.is_empty() {
                println!("{}", &proj_prompt);
                usr_prompts(
                    prompts.as_ref()
                ).await.unwrap();

                println!(" [0]: Select Prompt");

            } else {
                println!("This project has no prompts!");
                return Ok(AppState::AskModel)
            }
            println!(" [1]: Delete Prompt");
            println!(" [2]: Select Project");

            let choice: usize = usr_ask(" [ Select Option ] ").unwrap();
            match choice {
                0 => {
                    if !prompts.is_empty() {
                        let mut concat_prompts: Vec<String> = prompts
                            .iter()
                            .map(|p| format_prompt(p) )
                            .collect();

                        concat_prompts.push(proj_prompt);

                        let sel_p: String = item_selector(concat_prompts.clone()).unwrap().unwrap();
                        let sel_idx = concat_prompts.iter().position(|p| *p == sel_p).unwrap();
                        self.current_prompt = match prompts.get(sel_idx) {
                            Some(p) => Some(p.to_owned()),
                            _ => None,
                        };
                        return Ok(AppState::AskModel);
                    } else {
                        println!("Invalid input, try again.");
                    }
                },
                1 => {
                    if !prompts.is_empty() {
                        let concat_prompts: Vec<String> = prompts
                            .iter()
                            .map(|p| format_prompt(p) )
                            .collect();

                        let sel_p: String = item_selector(concat_prompts.clone()).unwrap().unwrap();
                        let sel_idx = concat_prompts.iter().position(|p| *p == sel_p).unwrap();
                        let del_prompt = match prompts.get(sel_idx) {
                            Some(p) => Some(p.to_owned()),
                            _ => None,
                        };

                        delete_prompt(pool, &del_prompt.unwrap())
                            .await
                            .expect("Unable to delete prompt");

                        return Ok(AppState::SelectPrompt);
                    } else {
                        println!("Invalid input, try again.");
                    }
                },
                2 => {
                    return Ok(AppState::SelectProject);
                },
                _ => {
                    println!("Invalid input, try again.");
                }
            }
        }
    }

    async fn handle_ask_model(
        &mut self,
        pool: &SqlitePool,
    ) -> Result<AppState> {
        loop {
            clear_screen();
            // Preparing scrolls 
            let scrolls = get_scrolls(
                pool,
                &self.current_project.as_ref().unwrap().project_id
            ).await.unwrap();

            usr_scrolls(
                pool,
                &self.current_project.as_ref().unwrap()
            ).await.unwrap();

            // Preparing prompts
            let prompt = self.current_prompt.as_ref();
            let file_prompt = fs::read_to_string(
                &PathBuf::from(
                    &self.current_project.as_ref().unwrap().project_path
                ).join("legatio.md")
            );

            let mut prompts: Option<Vec<Prompt>> = None;
            let mut pmp_chain: Option<Vec<Prompt>> = None;

            if !file_prompt.is_ok() {
                File::create(
                    &PathBuf::from(
                        &self.current_project.as_ref().unwrap().project_path
                    ).join("legatio.md")
                ).expect("Could not create file!");
            } else if prompt.is_some() {

                prompts = Some(get_prompts(
                    pool, 
                    &self.current_project.as_ref().unwrap().project_id
                ).await.unwrap());

                pmp_chain = Some(prompt_chain(
                    prompts.as_ref().unwrap().as_ref(),
                    &self.current_prompt.as_ref().unwrap()
                ));

                println!(" [ Prompt Chain ]");
                usr_prompt_chain(pmp_chain.as_ref().unwrap().as_ref());

            }

            // Menu
            println!(" [1] Ask the Model");
            println!(" [2] Switch branch");
            println!(" [3] Edit Scrolls");
            println!(" [4] Switch project");

            let choice = usr_ask(" [ Select Option ] ").unwrap();
            match choice  {
                1 => { 
                    let sys_prompt = system_prompt(&scrolls);
                    
                    if prompts.as_ref().is_none() {
                        prompts = Some(get_prompts(
                            pool, 
                            &self.current_project.as_ref().unwrap().project_id
                        ).await.unwrap());
                    }

                    if !prompts.as_ref().unwrap().is_empty() && prompt.is_some() {
                        pmp_chain = Some(prompt_chain(
                            &prompts.as_ref().unwrap().as_ref(),
                            prompt.unwrap()
                        ));
                    }

                    let curr_prompt = fs::read_to_string(
                        &PathBuf::from(
                            &self.current_project.as_ref().unwrap().project_path
                        ).join("legatio.md")
                    ).unwrap();

                    let output = get_openai_response(
                        &sys_prompt,
                        pmp_chain,
                        &curr_prompt
                    ).await.unwrap();
                    
                    let mut file = OpenOptions::new()
                            .write(true)
                            .append(true)
                            .open(&PathBuf::from(
                                &self.current_project
                                .as_ref()
                                .unwrap()
                                .project_path)
                            .join("legatio.md"))
                            .unwrap();
                    let out_md = format!("0o0o0o0o0 \n Answer: \n{}", output);
                    if let Err(e) = writeln!(file, "{}", out_md) {
                        eprintln!("Couldn't write to file: {}", e);
                    }
                    
                    let prev_id = match &self.current_prompt.as_ref() {
                        Some(p) => &p.prompt_id,
                        None => &self.current_project.as_ref().unwrap().project_id,
                    };

                    let mut lst_prompt = Prompt::new(
                        &self.current_project.as_ref().unwrap().project_id,
                        &curr_prompt,
                        &output,
                        &prev_id,
                    );
                    store_prompt(pool, &mut lst_prompt).await.unwrap();

                    self.current_prompt = Some(lst_prompt);
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
            clear_screen();
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
            println!("Select an option: \n [{}] Append a Scroll \n [{}] Remove a Scroll \n [{}] Select Prompt \n [{}] Select Project",
                scrolls.len(),
                scrolls.len() + 1,
                scrolls.len() + 2,
                scrolls.len() + 3,
            );

            let choice = usr_ask(" [ Select Option ]").unwrap();

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
                let scroll_idx = usr_ask("Select scroll index delete: ").unwrap();
                if scroll_idx < scrolls.len() {
                    delete_scroll(pool, &scrolls[scroll_idx].scroll_id).await.unwrap();
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
        let selected_scrolls = select_files(Some(&project.project_path)).unwrap();
        let scroll = read_file(&selected_scrolls, &project.project_id).unwrap();
        store_scroll(pool, &scroll).await.unwrap();
        Ok(get_scrolls(pool, &project.project_id).await.unwrap())
    }
    
}
