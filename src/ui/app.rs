use std::sync::Arc;

use serde::{Deserialize, Serialize};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    backend::Backend, layout::{Constraint, Direction, Layout}, text::Span, widgets::{Block, Borders, List}, DefaultTerminal, Frame
};
use skim::{Skim, SkimItem, SkimItemReceiver, SkimOptions, SkimOutput};
use sqlx::{Pool, Sqlite};

use crate::{
    db::{
        app_state::store_app_state, project::get_projects, 
        prompt::get_prompts_from_scroll, scroll::get_scrolls
    },
    utils::{
        file_utils::get_contents, logger::log_info, structs::Prompt
    }
};
use super::{
    drawer::{fetch_drawer_data, render_drawer, DrawerData},
    master::render_master_box, prompt_box::render_prompts_box,
};
//prompt_box::render_prompts_box


#[derive(Debug, Serialize, Deserialize)]
pub enum AppScreen {
    WelcomeScreen,
    ProjectSelection,
    ScrollSelection,
    HomeScreen,
    FindProjectSelection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    pub drawer_open: bool,
    pub selected_drawer_item: Option<String>, // Can be "Files" or "Scrolls"
    pub display_item_id: Option<String>, // Could represent either file_id or scroll_id
    pub input_mode: bool, // When true, user can input text in the main box
    pub highlighted_prompt_index: Option<usize>,
    pub project_id: Option<String>,
    pub scroll_id: Option<String>,
    pub should_exit: bool,
    #[serde(flatten)]
    pub state: AppScreen,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            drawer_open: true,
            selected_drawer_item: None,
            display_item_id: None,
            input_mode: false,
            highlighted_prompt_index: None,
            project_id: None,
            scroll_id: None,
            should_exit: false,
            state: AppScreen::WelcomeScreen,
        }
    }

    pub async fn run(&mut self, pool: &Pool<Sqlite>, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            match self.state {
                AppScreen::WelcomeScreen => {
                    terminal.draw(|f| { 
                        let _ = self.render_home_screen(f);
                    });
                }
                AppScreen::ProjectSelection => {
                    let projects = get_projects(pool).await.unwrap();
                    terminal.draw(|f| { 
                        //let _ = self.render_search(f, projects); 
                    });
                }
                AppScreen::FindProjectSelection => {
                    let home = String::from("/home/eduardoneville/Desktop/");
                    let dir_list = get_contents(&home, true, 5);
                    terminal.draw(|f| { 
                        //let _ = self.render_search(f, dir_list.unwrap());
                    });
                }
                AppScreen::ScrollSelection  => {
                    let scrolls = get_scrolls(pool, self.project_id.as_ref().unwrap()).await.unwrap();
                    terminal.draw(|f| { 
                        //let _ = self.render_search(f, scrolls);
                    });
                }
                AppScreen::HomeScreen => { 
                    let drawer_data = Some(fetch_drawer_data(self, pool).await);
                    let mut prompts = vec![];
                    if let Some((scroll_id, _)) = &drawer_data.as_ref().unwrap()
                        .scrolls
                        .iter()
                        .find(|(_, is_highlighted)| *is_highlighted)
                    {
                        prompts = get_prompts_from_scroll(pool, scroll_id).await.unwrap();
                    }

                    terminal.draw(|f| { 
                        let _ = self.render_ui(
                            f,
                            drawer_data.as_ref(),
                            prompts
                        );
                    });
                }
            }
        }

        if let Event::Key(key) = event::read()? {
            log_info("Key has been pressed");
            self.handle_key(key);
        };

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match self.state {
            AppScreen::WelcomeScreen => match key.code {
                KeyCode::Char('1') => {
                    self.state = AppScreen::ProjectSelection;
                }
                KeyCode::Char('2') => {
                    // Add your asynchronous fetching logic here if needed.
                    self.state = AppScreen::FindProjectSelection;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    // Exit and store the state asynchronously.
                    store_app_state(self);
                    self.should_exit = true;
                }
                _ => {}
            },
            AppScreen::ProjectSelection | AppScreen::FindProjectSelection => match key.code {
                KeyCode::Char(c) => {

                },
                KeyCode::Backspace => {
                },
                KeyCode::Esc => {
                    store_app_state(self);
                    self.should_exit = true;
                },
                _ => {}
            },
            AppScreen::HomeScreen => {
                
            }
            _ => {}
        }
    }

    fn render_ui(&mut self, f: &mut Frame, drawer_data: Option<&DrawerData>, prompts: Vec<Prompt>) {
        // Layout creation
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(if self.drawer_open { 25 } else { 0 }), // Drawer
                    Constraint::Percentage(50), // Main Box
                    Constraint::Percentage(25), // Prompts Box
                ]
                .as_ref(),
            )
            .split(f.area());

        // Render components (no `await` here since data is pre-fetched)
        if self.drawer_open {
            if let Some(data) = drawer_data {
                render_drawer(f, layout[0], data);
            }
        }
        render_master_box(f, layout[1], self);
        render_prompts_box(&self, f, layout[2], prompts);
    }

    fn render_home_screen(&mut self, f: &mut Frame) {
        let block = Block::default()
            .title("Legatio")
            .borders(Borders::ALL);
        let list = List::new(
            vec![
                Span::from("[1] Select a project"),
                Span::from("[2] Create a new one"),
            ]
        )
        .block(block);

        f.render_widget(list, f.area());
    }

    //fn render_search<T>(&mut self, f: &mut Frame, search_data: Vec<T>)
    //where
    //    T: std::fmt::Display + Clone + Send + Sync + 'static,
    //{
    //    // Prepare skim input
    //    let options = SkimOptions::default()
    //        .prompt(Some("Select an item: "))
    //        .build()
    //        .unwrap();

    //    let item_list: Vec<Arc<dyn SkimItem>> = search_data
    //        .into_iter()
    //        .map(|item| Arc::new(item.to_string()) as Arc<dyn SkimItem>)
    //        .collect();

    //    // The skim run function for interactive search
    //    let selected_items: Option<SkimOutput> =
    //        Skim::run_with(&options, Some(SkimItemReceiver::new(item_list.into_iter())));

    //    // Handle the user's selection
    //    if let Some(output) = selected_items {
    //        if let Some(selected_item) = output.selected_items.first() {
    //            println!("Selected Item: {}", selected_item.output()); // Print the user's chosen data
    //        }
    //    }
    //}

}

