use crossterm::{
    event::{self, Event, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::{backend::CrosstermBackend, Terminal};

use std::fs::{self, File};
use std::path::PathBuf;
use std::time::Duration;
use std::{io, vec};

use crate::{
    core::{
        canvas::{chain_into_canvas, chain_match_canvas},
        project::{
            build_select_project, delete_project, format_project_title, get_projects, store_project,
        },
        prompt::{
            delete_prompt, format_prompt, get_prompts, prompt_chain, store_prompt, system_prompt,
        },
        scroll::{delete_scroll, get_scrolls, read_file, store_scroll, update_scroll_content},
    },
    services::{
        display::{AppState, InputEvent},
        model::get_openai_response,
        search::{item_selector, select_files},
        ui::{usr_prompt_chain, usr_prompts, usr_scrolls},
    },
    utils::{
        logger::log_info,
        structs::{Project, Prompt},
    },
};
use anyhow::Result;
use sqlx::SqlitePool;

pub struct Legatio {
    state: AppState,
    current_project: Option<Project>,
    current_prompt: Option<Prompt>,
}

impl Default for Legatio {
    fn default() -> Self {
        Self::new()
    }
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
        // Initialize terminal with raw mode
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        let backend = CrosstermBackend::new(&mut stdout);
        let mut terminal = Terminal::new(backend)?;

        // Ensure we disable raw mode when application exits
        let result = self.main_loop(&mut terminal, pool).await;

        disable_raw_mode()?;
        terminal.show_cursor()?;
        result
    }

    async fn main_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<&mut io::Stdout>>,
        pool: &SqlitePool,
    ) -> Result<()> {
        // Fetch projects for initialization
        let projects = get_projects(pool).await.unwrap();
        if !projects.is_empty() {
            self.current_project = Some(projects[0].clone());
            self.state = AppState::SelectProject;
        }

        loop {
            self.draw(terminal, pool).await?;

            // Handle input
            let next_state = self.handle_input(pool).await?;
            self.state = next_state;
        }
    }

    async fn draw(
        &self,
        terminal: &mut Terminal<CrosstermBackend<&mut io::Stdout>>,
        pool: &SqlitePool,
    ) -> Result<()> {
        // Prepare all the data we might need to render
        //
        let top_title = format_project_title(&self.current_project);
        let top_text: Vec<Line>;
        let mut scroll_title: Option<&str> = None;
        let mut scroll_items: Option<Vec<Line>> = None;
        let mut bot_title = String::new();
        let mut bot_items: Vec<Line> = vec![];

        match self.state {
            // Define UI for specific state
            AppState::SelectProject => {
                top_text = vec![
                    Line::from("[s] Select Project"),
                    Line::from("[n] New Project"),
                    Line::from("[d] Delete Project"),
                    Line::from("[q] Quit"),
                ];
                bot_title = "[ Projects ]".to_string();

                let projects: Vec<Project> = get_projects(pool).await.unwrap();
                let (items, _) = build_select_project(&projects);
                bot_items.extend(items);
            }
            AppState::SelectPrompt => {
                top_text = vec![
                    Line::from("[s]: Select Prompt"),
                    Line::from("[d]: Delete Prompt"),
                    Line::from("[e]: Edit Scrolls"),
                    Line::from("[p]: Change Project"),
                    Line::from("[q]: Quit"),
                ];

                if let Some(project) = &self.current_project {
                    let project_name = project
                        .project_path
                        .split("/")
                        .last()
                        .unwrap_or("[Unnamed Project]");
                    bot_title = format!("[ {} -:- Prompts ]", project_name);

                    let prompts = get_prompts(pool, &project.project_id).await.unwrap();
                    if prompts.is_empty() {
                        bot_items.push(Line::from("This project has no prompts!"));
                    } else {
                        let prompt_strs = usr_prompts(prompts.as_ref()).await.unwrap();
                        for p in prompt_strs {
                            bot_items.push(Line::from(p));
                        }
                    }
                } else {
                    bot_items.push(Line::from("No active project"));
                }
            }
            AppState::AskModel => {
                top_text = vec![
                    Line::from("[a] Ask the Model"),
                    Line::from("[b] Switch branch"),
                    Line::from("[e] Edit Scrolls"),
                    Line::from("[p] Change Project"),
                    Line::from("[q] Quit"),
                ];
                scroll_title = Some("[ Scrolls ]");
                bot_title = String::from("[ Prompts ]");
                if let Some(project) = &self.current_project {
                    // Scroll PREP
                    let scrolls = usr_scrolls(pool, project).await.unwrap();
                    // Initialize `scroll_items` if it hasn't been initialized
                    if scroll_items.is_none() {
                        scroll_items = Some(vec![]);
                    }

                    // Now safely modify the inner `Vec<Line>`
                    if let Some(items) = scroll_items.as_mut() {
                        for scroll in scrolls {
                            items.push(Line::from(scroll)); // Mutable push happens here
                        }
                    }

                    // Prompt PREP
                    let prompt = self.current_prompt.as_ref();
                    let file_prompt = fs::read_to_string(
                        PathBuf::from(&self.current_project.as_ref().unwrap().project_path)
                            .join("legatio.md"),
                    );

                    let prompts: Option<Vec<Prompt>>;
                    let pmp_chain: Option<Vec<Prompt>>;
                    if file_prompt.is_err() {
                        File::create(
                            PathBuf::from(&self.current_project.as_ref().unwrap().project_path)
                                .join("legatio.md"),
                        )
                        .expect("Could not create file!");
                    } else if prompt.is_some() {
                        prompts = Some(get_prompts(pool, &project.project_id).await.unwrap());

                        if prompts.as_ref().unwrap().is_empty() {
                            bot_items.push(Line::from("This project has no prompts!"));
                        } else {
                            pmp_chain = Some(prompt_chain(
                                prompts.as_ref().unwrap().as_ref(),
                                self.current_prompt.as_ref().unwrap(),
                            ));

                            let p_strs = usr_prompt_chain(pmp_chain.as_ref().unwrap().as_ref());
                            p_strs
                                .iter()
                                .for_each(|p| bot_items.push(Line::from(p.to_string())));
                        }
                    }
                } else {
                    bot_items.push(Line::from("No active project"));
                }
            }
            AppState::EditScrolls => {
                top_text = vec![
                    Line::from("[n] New Scroll"),
                    Line::from("[d] Delete Scroll"),
                    Line::from("[a] Ask Model"),
                    Line::from("[s] Switch Branch"),
                    Line::from("[p] Change Project"),
                    Line::from("[q]: Quit"),
                ];
                bot_title = "[ Scrolls ]".to_string();

                if let Some(project) = &self.current_project {
                    let scrolls = get_scrolls(pool, &project.project_id)
                        .await
                        .unwrap_or_default();
                    for scroll in scrolls.iter() {
                        let scroll_name =
                            match scroll.scroll_path.strip_prefix(&project.project_path) {
                                Some(remaining) => {
                                    remaining.strip_prefix('/').unwrap_or(remaining).to_string()
                                }
                                None => scroll.scroll_path.to_string(),
                            };
                        bot_items.push(Line::from(scroll_name));
                    }
                }
            }
        }

        // Call render function with prepared data
        self.render(
            terminal,
            &top_title,
            &top_text,
            scroll_title,
            scroll_items,
            &bot_title,
            &bot_items,
        )
    }

    fn render(
        &self,
        terminal: &mut Terminal<CrosstermBackend<&mut io::Stdout>>,
        top_title: &str,
        top_text: &[Line],
        scroll_title: Option<&str>,
        scroll_text: Option<Vec<Line>>,
        bot_title: &str,
        bot_items: &[Line],
    ) -> Result<()> {
        // Top box
        let top_box = Paragraph::new(top_text.to_owned())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .title(top_title),
            )
            .style(Style::default().fg(Color::LightBlue));

        // Scroll box
        let (scroll_box, constraints) =
            if let (Some(title), Some(text)) = (scroll_title, scroll_text.as_ref()) {
                // Both `scroll_title` and `scroll_text` exist, so create the scroll box
                let scroll_box = Paragraph::new(text.clone())
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Thick)
                            .title(title),
                    )
                    .style(Style::default().fg(Color::LightBlue));

                let constraints = Vec::from([
                    Constraint::Percentage(18),
                    Constraint::Percentage(21),
                    Constraint::Percentage(61),
                ]);

                (Some(scroll_box), constraints)
            } else {
                // No scroll box; provide default constraints
                (
                    None,
                    Vec::from([Constraint::Percentage(18), Constraint::Percentage(82)]),
                )
            };

        // Bottom box
        let bot_box = Paragraph::new(bot_items.to_owned())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .title(bot_title),
            )
            .style(Style::default().fg(Color::LightBlue));

        // Terminal draw
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(f.area());

            // Render the top box
            f.render_widget(top_box, chunks[0]);

            // Render the scroll box if it exists
            if let Some(scroll_box) = scroll_box {
                f.render_widget(scroll_box, chunks[1]);
                f.render_widget(bot_box, chunks[2]);
            } else {
                f.render_widget(bot_box, chunks[1]);
            }
        })?;

        Ok(())
    }

    async fn handle_input(&mut self, pool: &SqlitePool) -> Result<AppState> {
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                return match self.state {
                    AppState::SelectProject => {
                        self.process_select_project_input(key_event, pool).await
                    }
                    AppState::SelectPrompt => {
                        self.process_select_prompt_input(key_event, pool).await
                    }
                    AppState::AskModel => self.process_ask_model_input(key_event, pool).await,
                    AppState::EditScrolls => self.process_edit_scrolls_input(key_event, pool).await,
                };
            }
        }
        Ok(self.state)
    }

    async fn process_select_project_input(
        &mut self,
        key_event: KeyEvent,
        pool: &SqlitePool,
    ) -> Result<AppState> {
        match InputEvent::from(key_event) {
            InputEvent::Select => {
                // Fetch all projects
                let projects = get_projects(pool).await.unwrap();
                if !projects.is_empty() {
                    let (_, str_names) = build_select_project(&projects);
                    if let Some(selected_project) = item_selector(str_names.clone()).unwrap() {
                        let sel_idx = str_names
                            .iter()
                            .position(|p| *p == selected_project)
                            .unwrap();
                        self.current_project = Some(projects[sel_idx].clone());
                        return Ok(AppState::SelectPrompt);
                    } else {
                        enable_raw_mode()?;
                        return Ok(AppState::SelectProject);
                    }
                } else {
                    let selected_dir = select_files(None).unwrap().unwrap();
                    let project = Project::new(&selected_dir);
                    store_project(pool, &project).await.unwrap();
                    self.current_project = Some(project.clone());
                    return Ok(AppState::EditScrolls);
                }
            }
            InputEvent::New => {
                let selected_dir = select_files(None).unwrap().unwrap();
                let project = Project::new(&selected_dir);
                store_project(pool, &project).await.unwrap();
                self.current_project = Some(project.clone());
                return Ok(AppState::EditScrolls);
            }
            InputEvent::Delete => {
                let projects = get_projects(pool).await.unwrap();
                if !projects.is_empty() {
                    let (_, str_names) = build_select_project(&projects);
                    if let Some(selected_project) = item_selector(str_names.clone()).unwrap() {
                        let sel_idx = str_names
                            .iter()
                            .position(|p| *p == selected_project)
                            .unwrap();

                        delete_project(pool, &projects[sel_idx].project_id)
                            .await
                            .unwrap();
                    } else {
                        return Ok(AppState::SelectProject);
                    }
                }
                return Ok(AppState::SelectProject);
            }
            InputEvent::Quit => {
                disable_raw_mode()?;
                std::process::exit(0);
            }
            _ => {}
        }
        Ok(AppState::SelectProject)
    }

    // Handles user input when selecting a prompt
    async fn process_select_prompt_input(
        &mut self,
        key_event: KeyEvent,
        pool: &SqlitePool,
    ) -> Result<AppState> {
        match InputEvent::from(key_event) {
            InputEvent::Select => {
                if let Some(project) = &self.current_project {
                    let prompts = get_prompts(pool, &project.project_id).await.unwrap();
                    if !prompts.is_empty() {
                        let project_name = project
                            .project_path
                            .split('/')
                            .next_back()
                            .unwrap_or("[Unnamed Project]");
                        let mut concat_prompts =
                            vec![format!(" -[ {} -:- Unchained]-", project_name)];
                        for p in prompts.iter() {
                            let (p_str, o_str) = format_prompt(p);
                            concat_prompts.push(format!("{}\n{}", p_str, o_str));
                        }
                        concat_prompts.reverse();

                        if let Some(selected_prompt) =
                            item_selector(concat_prompts.clone()).unwrap()
                        {
                            let mut idx = concat_prompts
                                .iter()
                                .position(|p| p == &selected_prompt)
                                .unwrap();

                            log_info(&format!(
                                "Selected: {:?} | idx: {} | len: {}",
                                selected_prompt,
                                idx,
                                prompts.len()
                            ));

                            if idx < prompts.len() {
                                idx = prompts.len() - 1 - idx;
                                self.current_prompt = prompts.get(idx).map(|p| p.to_owned());
                                chain_into_canvas(
                                    project,
                                    Some(&prompts),
                                    self.current_prompt.as_ref(),
                                )
                                .unwrap();
                            } else {
                                self.current_prompt = None;
                                chain_into_canvas(project, None, None).unwrap();
                            }
                        } else {
                            enable_raw_mode()?;
                            return Ok(AppState::SelectPrompt);
                        }
                    }
                    return Ok(AppState::AskModel);
                }
            }
            InputEvent::Delete => {
                if let Some(project) = &self.current_project {
                    let prompts = get_prompts(pool, &project.project_id).await.unwrap();
                    let project_name = project
                        .project_path
                        .split('/')
                        .next_back()
                        .unwrap_or("[Unnamed Project]");

                    let mut concat_prompts = vec![format!(" -[ {} -:- Unchained]-", project_name)];
                    for p in prompts.iter() {
                        let (p_str, o_str) = format_prompt(p);
                        concat_prompts.push(format!("{}\n{}", p_str, o_str));
                    }

                    if let Some(selected_prompt) = item_selector(concat_prompts.clone()).unwrap() {
                        let index = concat_prompts
                            .iter()
                            .position(|p| p == &selected_prompt)
                            .unwrap();
                        delete_prompt(pool, &prompts[index]).await.unwrap();
                    } else {
                        return Ok(AppState::SelectPrompt);
                    }
                }
                return Ok(AppState::SelectPrompt);
            }
            InputEvent::ChangeProject => {
                return Ok(AppState::SelectProject);
            }
            InputEvent::EditScrolls => {
                return Ok(AppState::EditScrolls);
            }
            InputEvent::Quit => {
                //TODO handle store appstate
                disable_raw_mode()?;
                std::process::exit(0);
            }
            _ => {}
        }
        Ok(AppState::SelectPrompt)
    }

    // Handles user input when asking the model for a response
    async fn process_ask_model_input(
        &mut self,
        key_event: KeyEvent,
        pool: &SqlitePool,
    ) -> Result<AppState> {
        match InputEvent::from(key_event) {
            InputEvent::AskModel => {
                if let Some(project) = &self.current_project {
                    let scrolls = get_scrolls(pool, &project.project_id).await.unwrap();
                    let mut new_scrolls = Vec::new();
                    for scroll in scrolls.iter() {
                        let new_scroll = update_scroll_content(pool, scroll).await.unwrap();
                        new_scrolls.push(new_scroll);
                    }
                    let sys_prompt = system_prompt(&new_scrolls).await;

                    let prompts = get_prompts(pool, &project.project_id).await.unwrap();

                    let mut chain: Option<Vec<Prompt>> = None;
                    if let Some(curr_prompt) = &self.current_prompt {
                        chain = Some(prompt_chain(&prompts, curr_prompt));
                    }
                    log_info(&format!("Curr prompt: {:?}", self.current_prompt.as_ref()));

                    let final_prompt = chain_match_canvas(project).unwrap();
                    log_info(&format!("Question: {}", final_prompt));

                    let output = get_openai_response(&sys_prompt, chain, &final_prompt)
                        .await
                        .unwrap();

                    let new_prompt = Prompt::new(
                        &project.project_id,
                        &final_prompt,
                        &output,
                        &self
                            .current_prompt
                            .as_ref()
                            .map_or(project.project_id.clone(), |p| p.prompt_id.clone()),
                    );

                    store_prompt(pool, &new_prompt).await.unwrap();
                    self.current_prompt = Some(new_prompt);

                    log_info(&format!("New prompt: {:?}", self.current_prompt.as_ref()));
                    let mut new_prompts = prompts.clone();
                    new_prompts.push(self.current_prompt.as_ref().unwrap().clone());

                    chain_into_canvas(project, Some(&new_prompts), self.current_prompt.as_ref())
                        .unwrap();
                }
            }
            InputEvent::SwitchBranch => {
                return Ok(AppState::SelectPrompt);
            }
            InputEvent::EditScrolls => {
                return Ok(AppState::EditScrolls);
            }
            InputEvent::ChangeProject => {
                return Ok(AppState::SelectProject);
            }
            InputEvent::Quit => {
                disable_raw_mode()?;
                std::process::exit(0);
            }
            _ => {}
        }
        Ok(AppState::AskModel)
    }

    // Handles user input when editing scrolls
    async fn process_edit_scrolls_input(
        &mut self,
        key_event: KeyEvent,
        pool: &SqlitePool,
    ) -> Result<AppState> {
        match InputEvent::from(key_event) {
            InputEvent::New => {
                if let Some(project) = &self.current_project {
                    disable_raw_mode()?;
                    let selected_scrolls =
                        select_files(Some(&project.project_path)).unwrap().unwrap();
                    enable_raw_mode()?;
                    let new_scroll =
                        read_file(&selected_scrolls, &project.project_id, None).unwrap();
                    store_scroll(pool, &new_scroll).await.unwrap();
                }
                return Ok(AppState::EditScrolls);
            }
            InputEvent::Delete => {
                if let Some(project) = &self.current_project {
                    let scrolls = get_scrolls(pool, &project.project_id).await.unwrap();
                    let scroll_names = scrolls
                        .iter()
                        .map(|s| s.scroll_path.clone())
                        .collect::<Vec<_>>();

                    disable_raw_mode()?;
                    if let Some(selected_scroll) = item_selector(scroll_names.clone()).unwrap() {
                        enable_raw_mode()?;
                        let idx = scroll_names
                            .iter()
                            .position(|s| s == &selected_scroll)
                            .unwrap();

                        if idx < scrolls.len() {
                            delete_scroll(pool, &scrolls[idx].scroll_id).await.unwrap();
                        }
                    } else {
                        enable_raw_mode()?;
                        return Ok(AppState::EditScrolls);
                    }
                }
                return Ok(AppState::EditScrolls);
            }
            InputEvent::Select => {
                return Ok(AppState::SelectPrompt);
            }
            InputEvent::ChangeProject => {
                return Ok(AppState::SelectProject);
            }
            InputEvent::AskModel => {
                return Ok(AppState::AskModel);
            }
            InputEvent::Quit => {
                disable_raw_mode()?;
                std::process::exit(0);
            }
            _ => {}
        }
        Ok(AppState::EditScrolls)
    }
}
