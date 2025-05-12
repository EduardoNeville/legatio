use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::{backend::CrosstermBackend, Terminal};

use std::fs::{self, File};
use std::path::PathBuf;
use std::{io, vec};

use crate::{
    core::{
        canvas::{chain_into_canvas, chain_match_canvas},
        project::{
            build_select_project,
            delete_project,
            format_project_title,
            get_projects,
            store_project,
        },
        prompt::{
            delete_prompt,
            format_prompt,
            get_prompts,
            prompt_chain,
            store_prompt,
            system_prompt,
        },
        scroll::{
            delete_scroll,
            get_scrolls,
            read_file,
            store_scroll,
            update_scroll_content
        },
    },
    services::{
        config::{read_config, store_config, UserConfig},
        //model::{ask_question, Question, LLM},
        search::{item_selector, select_files, select_directories},
        ui::{extract_theme_colors, usr_prompt_chain, usr_prompts, usr_scrolls},
    },
    utils::structs::{Project, Prompt, Scroll},
};

use anyhow::Result;
use ask_ai::{
    ask_ai::ask_question,
    config::{AiConfig, AiPrompt, Framework, Question},
};
use sqlx::SqlitePool;

pub struct Legatio {
    state: AppState,
    current_project: Option<Project>,
    current_prompt: Option<Prompt>,
    user_config: Option<UserConfig>,
    project_list_cache: Option<Vec<Project>>,
    prompt_list_cache: Option<Vec<Prompt>>,
    scroll_list_cache: Option<Vec<Scroll>>,
}

#[derive(Clone, Copy)]
enum AppState {
    SelectProject,
    SelectPrompt,
    AskModel,
    EditScrolls,
    AskModelConfirmation,
    Quit,
}

#[derive(Debug, PartialEq, Eq)]
enum InputEvent {
    Select,
    New,
    Delete,
    SwitchBranch,
    ChangeProject,
    EditScrolls,
    AskModel,
    Quit,
    Confirm,
    Cancel,
    NoOp,
}

impl Default for Legatio {
    fn default() -> Self {
        Self::new()
    }
}

/// The `Legatio` library is a command-line tool designed to facilitate the management and interaction with AI-driven projects.
/// It provides an interface for managing projects, creating and organizing prompts (instructions for AI models),
/// handling text-based assets (known as scrolls), and interacting with AI models for output generation.
/// The tool is particularly aimed at software engineers or anyone who wants to structure workflows involving external AI model interactions.
///
/// ## Overview
/// The main struct in this library is `Legatio`, which acts as the core entry point.
/// Through `Legatio`, you can perform a wide variety of operations, such as:
///
/// - Select and manage projects.
/// - Manage and edit scrolls and prompts.
/// - Chain prompts together.
/// - Ask questions to AI models (like OpenAI's ChatGPT).
/// - Render a rich terminal user interface (using `ratatui` and `crossterm`).
/// - Confirm actions to prevent accidental execution of AI queries.
///
/// The library relies on an underlying SQLite database (managed via `sqlx`) to persist projects, prompts, and scrolls.
///
/// ## Features
///
/// - **Keyboard-based navigation:** Navigate through various actions and states using keyboard shortcuts.
/// - **Project management:** Manage multiple projects with unique scrolls and prompts.
/// - **Rich UI Rendering:** Leverage `ratatui` for creating visually appealing terminal widgets.
/// - **AI Support:** Query AI models for generating new outputs or augmenting workflows.
/// - **Configurable AI Models:** Configure the AI backend (`Framework`) and provide custom parameters like models and tokens.
///
/// ## Data Structure
/// The library organizes its data as follows:
///
/// - **Project:** Represents a self-contained entity comprising scrolls and prompts.
/// - **Scrolls:** Static files or content associated with a project served as context for AI prompts.
/// - **Prompts:** Instructions to be used in interacting with the AI model.
/// - **Configuration:** User-level settings, including AI model settings and UI themes.
///
impl Legatio {
    /// Creates a new instance of the `Legatio` application.
    ///
    /// This initializes the application with a default state (`AppState::SelectProject`) and sets
    /// placeholders for the active project, active prompt, and user configuration.
    pub fn new() -> Self {
        Legatio {
            state: AppState::SelectProject,
            current_project: None,
            current_prompt: None,
            user_config: None,
            project_list_cache: None,
            prompt_list_cache: None,
            scroll_list_cache: None,
        }
    }

    /// The main entry point for running the `Legatio` application.
    ///
    /// This function:
    /// 1. Sets up a terminal interface with raw mode.
    /// 2. Initializes the application state.
    /// 3. Runs a continuous loop (`main_loop`) until the application terminates.
    /// 4. Ensures raw mode is disabled on exit.
    ///
    /// ### Arguments:
    /// `pool` - A `SqlitePool` connection to the underlying SQLite database.
    ///
    /// ### Returns:
    /// - `Result<()>` indicating success or failure.
    pub async fn run(&mut self, pool: &SqlitePool) -> Result<()> {
        // Initialize terminal with raw mode
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        let backend = CrosstermBackend::new(&mut stdout);
        let mut terminal = Terminal::new(backend)?;

        // Default config for user
        let default_config = UserConfig {
            ai_conf: AiConfig {
                llm: Framework::OpenAI,
                model: String::from("chatgpt-4o-latest"),
                max_token: None,
            },
            theme: String::from("Tokyo Storm"),
            ask_conf: true,
        };

        self.user_config = Some(read_config().unwrap_or(default_config));
        store_config(self.user_config.as_ref().unwrap()).unwrap();

        // Run the main loop
        let result = self.main_loop(&mut terminal, pool).await;

        // Clean up terminal
        disable_raw_mode()?;
        terminal.show_cursor()?;
        result
    }

    /// The primary loop handling all core application functionality and state transitions.
    ///
    /// Contains the following key elements:
    /// 1. **State Management:** Switch between states like project selection, prompt selection, and AI interaction.
    /// 2. **Rendering:** Dynamically updates terminal UI based on the current state.
    /// 3. **Input Handling:** Processes user input and react accordingly.
    ///
    /// ### Arguments:
    /// `terminal` - A mutable reference to the `ratatui` terminal instance.
    /// `pool` - A `SqlitePool` connection to the SQLite database.
    ///
    /// ### Returns:
    /// - `Result<()>` indicating success or failure.
    async fn main_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<&mut io::Stdout>>,
        pool: &SqlitePool,
    ) -> Result<()> {
        // Fetch all projects from cache
        let projects = if let Some(cache) = &self.project_list_cache {
            cache.clone()
        } else {
            let p = get_projects(pool).await?;
            self.project_list_cache = Some(p.clone());
            p
        };
        if !projects.is_empty() {
            self.current_project = Some(projects[0].clone());
            self.state = AppState::SelectProject;
        }

        // Initial draw to display the UI
        self.draw(terminal, pool).await?;

        // Main event loop: wait for key events
        loop {
            // Block until a key event is received
            if let Event::Key(key_event) = event::read()? {
                // Process the input
                let next_state = self.handle_input_with_key(pool, key_event).await?;
                self.state = next_state;

                // Redraw the UI after handling input
                self.draw(terminal, pool).await?;

                // Exit if the state is Quit
                if matches!(self.state, AppState::Quit) {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Draws the current state of the application onto the terminal screen.
    ///
    /// This method renders different UI elements (project list, scroll details, prompt chains,
    /// confirmation popups, etc.) depending on the application's state.
    ///
    /// ### Arguments:
    /// `terminal` - The `ratatui` terminal where widgets will be rendered.
    /// `pool` - The database connection pool to fetch data (if needed).
    ///
    /// ### Returns:
    /// - `Result<()>` on success.
    ///
    /// ### Remarks:
    /// This method is highly state-dependent and processes different inputs in different states.
    async fn draw(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<&mut io::Stdout>>,
        pool: &SqlitePool,
    ) -> Result<()> {
        // Prepare all the data we might need to render
        let theme = &self.user_config.as_ref().unwrap().theme;
        let colors = extract_theme_colors(theme)?;
        let primary_color = colors.primary;
        let secondary_color = colors.secondary;
        let accent_color = colors.accent;

        let top_title = format_project_title(&self.current_project);
        let top_text: Vec<Line>;
        let mut scroll_title: Option<&str> = None;
        let mut scroll_text: Option<Vec<Line>> = None;
        let mut bot_title = String::new();
        let mut bot_items: Vec<Line> = vec![];
        let mut pop_up = false;

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

                // Fetch all projects from cache
                let projects = if let Some(cache) = &self.project_list_cache {
                    cache.clone()
                } else {
                    let p = get_projects(pool).await?;
                    self.project_list_cache = Some(p.clone());
                    p
                };
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

                    // Fetch all prompts from cache
                    let prompts = if let Some(cache) = &self.prompt_list_cache {
                        cache.clone()
                    } else {
                        let p = get_prompts(pool, &project.project_id).await?;
                        self.prompt_list_cache = Some(p.clone());
                        p
                    };

                    if prompts.is_empty() {
                        bot_items.push(Line::from("This project has no prompts!"));
                    } else {
                        let prompt_strs = usr_prompts(prompts.as_ref()).await?;
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
                    let scrolls = usr_scrolls(pool, project).await?;
                    // Initialize `scroll_text` if it hasn't been initialized
                    if scroll_text.is_none() {
                        scroll_text = Some(vec![]);
                    }

                    // Now safely modify the inner `Vec<Line>`
                    if let Some(items) = scroll_text.as_mut() {
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

                    let pmp_chain: Option<Vec<Prompt>>;
                    if file_prompt.is_err() {
                        File::create(
                            PathBuf::from(&self.current_project.as_ref().unwrap().project_path)
                                .join("legatio.md"),
                        )
                        .expect("Could not create file!");
                    } else if prompt.is_some() {
                        // Fetch all prompts from cache
                        let prompts = if let Some(cache) = &self.prompt_list_cache {
                            cache.clone()
                        } else {
                            let p = get_prompts(pool, &project.project_id).await?;
                            self.prompt_list_cache = Some(p.clone());
                            p
                        };

                        if prompts.is_empty() {
                            bot_items.push(Line::from("This project has no prompts!"));
                        } else {
                            pmp_chain = Some(prompt_chain(
                                prompts.as_ref(),
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
                    // Fetch all prompts from cache
                    let scrolls: Vec<Scroll> = if let Some(cache) = &self.scroll_list_cache {
                        cache.clone()
                    } else {
                        let s = get_scrolls(pool, &project.project_id).await?;
                        self.scroll_list_cache = Some(s.clone());
                        s
                    };
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
            AppState::AskModelConfirmation => {
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
                    let scrolls = usr_scrolls(pool, project).await?;
                    // Initialize `scroll_text` if it hasn't been initialized
                    if scroll_text.is_none() {
                        scroll_text = Some(vec![]);
                    }

                    // Now safely modify the inner `Vec<Line>`
                    if let Some(items) = scroll_text.as_mut() {
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

                    let pmp_chain: Option<Vec<Prompt>>;
                    if file_prompt.is_err() {
                        File::create(
                            PathBuf::from(&self.current_project.as_ref().unwrap().project_path)
                                .join("legatio.md"),
                        )
                        .expect("Could not create file!");
                    } else if prompt.is_some() {
                        // Fetch all prompts from cache
                        let prompts = if let Some(cache) = &self.prompt_list_cache {
                            cache.clone()
                        } else {
                            let p = get_prompts(pool, &project.project_id).await?;
                            self.prompt_list_cache = Some(p.clone());
                            p
                        };

                        if prompts.is_empty() {
                            bot_items.push(Line::from("This project has no prompts!"));
                        } else {
                            pmp_chain = Some(prompt_chain(
                                prompts.as_ref(),
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
                pop_up = true;
            }
            // TODO: is this correct?
            AppState::Quit => return Ok(())
        }

        // Call render function with prepared data
        self.render(
            terminal,
            &top_title,
            &top_text,
            scroll_title,
            scroll_text,
            &bot_title,
            &bot_items,
            primary_color,
            secondary_color,
            accent_color,
            pop_up,
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
        primary_color: Color,
        secondary_color: Color,
        accent_color: Color,
        pop_up: bool,
    ) -> Result<()> {
        // Top box
        let top_box = if pop_up {
            Paragraph::new(vec![Line::from("[y]es [n]o")])
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .style(Style::default().fg(accent_color))
                        .title("[ Confirm ]"),
                )
                .style(Style::default().fg(secondary_color))
        } else {
            Paragraph::new(top_text.to_owned())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .style(Style::default().fg(primary_color))
                        .title(top_title),
                )
                .style(Style::default().fg(secondary_color))
        };

        // Scroll box
        let (scroll_box, constraints) =
            if let (Some(title), Some(text)) = (scroll_title, scroll_text.as_ref()) {
                // Both `scroll_title` and `scroll_text` exist, so create the scroll box
                let scroll_box = Paragraph::new(text.clone())
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Thick)
                            .style(Style::default().fg(primary_color))
                            .title(title),
                    )
                    .style(Style::default().fg(secondary_color));

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
                    .style(Style::default().fg(accent_color))
                    .title(bot_title),
            )
            .style(Style::default().fg(secondary_color));

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

    fn state_specific_keys(&self, key_event: KeyEvent) -> InputEvent {
        match self.state {
            AppState::SelectProject => match key_event {
                KeyEvent {
                    code: KeyCode::Char('s'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::Select,
                KeyEvent {
                    code: KeyCode::Char('n'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::New,
                KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::Delete,
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::Quit,
                _ => InputEvent::NoOp,
            },
            AppState::SelectPrompt => match key_event {
                KeyEvent {
                    code: KeyCode::Char('s'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::Select,
                KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::Delete,
                KeyEvent {
                    code: KeyCode::Char('p'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::ChangeProject,
                KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::EditScrolls,
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::Quit,
                _ => InputEvent::NoOp,
            },
            AppState::AskModel => match key_event {
                KeyEvent {
                    code: KeyCode::Char('a'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::AskModel,
                KeyEvent {
                    code: KeyCode::Char('p'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::ChangeProject,
                KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::EditScrolls,
                KeyEvent {
                    code: KeyCode::Char('b'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::SwitchBranch,
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::Quit,
                _ => InputEvent::NoOp,
            },
            AppState::EditScrolls => match key_event {
                KeyEvent {
                    code: KeyCode::Char('n'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::New,
                KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::Delete,
                KeyEvent {
                    code: KeyCode::Char('s'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::SwitchBranch,
                KeyEvent {
                    code: KeyCode::Char('p'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::ChangeProject,
                KeyEvent {
                    code: KeyCode::Char('a'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::AskModel,
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                    ..
                } => InputEvent::Quit,
                _ => InputEvent::NoOp,
            },
            AppState::AskModelConfirmation => match key_event {
                KeyEvent {
                    code: KeyCode::Char('y'),
                    ..
                } => InputEvent::Confirm,
                KeyEvent {
                    code: KeyCode::Char('n'),
                    ..
                } => InputEvent::Cancel,
                _ => InputEvent::NoOp,
            },
            AppState::Quit => InputEvent::Quit,
        }
    }

    /// Handles user input events (like keypresses) based on the application's current state.
    ///
    /// ### Input Handling:
    /// Depending on the current application state (project selection, prompt interaction),
    /// user inputs are processed accordingly.
    ///
    /// ### Arguments:
    /// `pool` - The database connection pool to interact with stored data.
    /// `key_event` Event that will trigger a change in the screen
    ///
    /// ### Returns:
    /// - `Result<AppState>`: Returns the next `AppState` after processing the input.
    async fn handle_input_with_key(
        &mut self,
        pool: &SqlitePool,
        key_event: KeyEvent,
    ) -> Result<AppState> {
        let input_event = self.state_specific_keys(key_event); // Get state-specific keys

        match self.state {
            AppState::SelectProject => self.process_select_project_input(input_event, pool).await,
            AppState::SelectPrompt => self.process_select_prompt_input(input_event, pool).await,
            AppState::AskModel => self.process_ask_model_input(input_event, pool).await,
            AppState::EditScrolls => self.process_edit_scrolls_input(input_event, pool).await,
            AppState::AskModelConfirmation => {
                self.process_confirmation_popup_input(input_event, pool).await
            }
            AppState::Quit => Ok(AppState::Quit)
        }
    }

    /// Processes user input when the application is in the `AppState::SelectProject` state.
    ///
    /// In this state, users can:
    /// - Select an existing project.
    /// - Create a new project.
    /// - Delete a project.
    /// - Exit the application.
    ///
    /// ### Arguments:
    /// `key_event` - The user input event.
    /// `pool` - The database connection pool.
    ///
    /// ### Returns:
    /// - `Result<AppState>`: Returns the next state of the application.
    async fn process_select_project_input(
        &mut self,
        key_event: InputEvent,
        pool: &SqlitePool,
    ) -> Result<AppState> {
        match key_event {
            InputEvent::Select => {
                // Fetch all projects from cache
                let projects = if let Some(cache) = &self.project_list_cache {
                    cache.clone()
                } else {
                    let p = get_projects(pool).await?;
                    self.project_list_cache = Some(p.clone());
                    p
                };
                if !projects.is_empty() {
                    let (_, str_names) = build_select_project(&projects);
                    if let Some(selected_project) = item_selector(str_names.clone())? {
                        let sel_idx = str_names
                            .iter()
                            .position(|p| *p == selected_project)
                            .unwrap();
                        self.current_project = Some(projects[sel_idx].clone());

                        // Clear cache for new project
                        self.scroll_list_cache = None;
                        self.prompt_list_cache = None;
                        return Ok(AppState::SelectPrompt);
                    } else {
                        enable_raw_mode()?;
                        return Ok(AppState::SelectProject);
                    }
                } else {
                    let selected_dir = select_directories(None).unwrap().unwrap();
                    let project = Project::new(&selected_dir);
                    store_project(pool, &project).await?;
                    self.current_project = Some(project.clone());
                    // Clear cache
                    self.project_list_cache = Some(vec![project]);
                    // Clear cache for new project
                    self.scroll_list_cache = None;
                    self.prompt_list_cache = None;
                    return Ok(AppState::EditScrolls);
                }
            }
            InputEvent::New => {
                let selected_dir = select_directories(None).unwrap().unwrap();
                // Fetch all projects from cache
                let projects = if let Some(cache) = &self.project_list_cache {
                    cache.clone()
                } else {
                    let p = get_projects(pool).await?;
                    self.project_list_cache = Some(p.clone());
                    p
                };
                let old_proj = projects.iter().find(|p| p.project_path == selected_dir);
                if old_proj.is_some() {
                    self.current_project = Some(old_proj.unwrap().to_owned());
                } else {
                    let project = Project::new(&selected_dir);
                    store_project(pool, &project).await?;
                    self.current_project = Some(project.to_owned());
                    // Clear project cache
                    self.project_list_cache = None;
                }
                // Clear cache for new project
                self.scroll_list_cache = None;
                self.prompt_list_cache = None;
                return Ok(AppState::EditScrolls);
            }
            InputEvent::Delete => {
                // Fetch all projects from cache
                let projects = if let Some(cache) = &self.project_list_cache {
                    cache.clone()
                } else {
                    let p = get_projects(pool).await?;
                    self.project_list_cache = Some(p.clone());
                    p
                };
                if !projects.is_empty() {
                    let (_, str_names) = build_select_project(&projects);
                    if let Some(selected_project) = item_selector(str_names.clone())? {
                        let sel_idx = str_names
                            .iter()
                            .position(|p| *p == selected_project)
                            .unwrap();

                        delete_project(pool, &projects[sel_idx].project_id).await?;
                        // Clear the cache
                        self.project_list_cache = None;
                    } else {
                        return Ok(AppState::SelectProject);
                    }
                }
                return Ok(AppState::SelectProject);
            }
            InputEvent::Quit => Ok(AppState::Quit),
            _ => Ok(AppState::SelectProject),
        }
    }

    /// Processes user input in the `AppState::SelectPrompt` state.
    ///
    /// In this state, users can:
    /// - Browse and select prompts.
    /// - Delete prompts.
    /// - Edit scrolls.
    /// - Change the active project.
    /// - Exit the application.
    ///
    /// ### Arguments:
    /// `key_event` - The user input event.
    /// `pool` - The database connection pool.
    ///
    /// ### Returns:
    /// - `Result<AppState>`: Returns the next state of the application.
    async fn process_select_prompt_input(
        &mut self,
        key_event: InputEvent,
        pool: &SqlitePool,
    ) -> Result<AppState> {
        match key_event {
            InputEvent::Select => {
                if let Some(project) = &self.current_project {
                    // Fetch all prompts from cache
                    let prompts = if let Some(cache) = &self.prompt_list_cache {
                        cache.clone()
                    } else {
                        let p = get_prompts(pool, &project.project_id).await?;
                        self.prompt_list_cache = Some(p.clone());
                        p
                    };

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

                        if let Some(selected_prompt) = item_selector(concat_prompts.clone())? {
                            let mut idx = concat_prompts
                                .iter()
                                .position(|p| p == &selected_prompt)
                                .unwrap();

                            if idx < prompts.len() {
                                idx = prompts.len() - 1 - idx;
                                self.current_prompt = prompts.get(idx).map(|p| p.to_owned());
                                chain_into_canvas(
                                    project,
                                    Some(&prompts),
                                    self.current_prompt.as_ref(),
                                )?;
                            } else {
                                self.current_prompt = None;
                                chain_into_canvas(project, None, None)?;
                            }
                        } else {
                            enable_raw_mode()?;
                            return Ok(AppState::SelectPrompt);
                        }
                    } else {
                        // No prompts so only place holder
                        chain_into_canvas(project, None, None)?;
                    }
                    return Ok(AppState::AskModel);
                }
            }
            InputEvent::Delete => {
                if let Some(project) = &self.current_project {
                    let prompts = get_prompts(pool, &project.project_id).await?;
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

                    if let Some(selected_prompt) = item_selector(concat_prompts.clone())? {
                        let index = concat_prompts
                            .iter()
                            .position(|p| p == &selected_prompt)
                            .unwrap();
                        delete_prompt(pool, &prompts[index]).await?;

                        // Clear the cache
                        self.prompt_list_cache = None;
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

    /// Processes user input in the `AppState::AskModel` state.
    ///
    /// In this state, users can:
    /// - Send the current prompt (and its associated chain) to an AI model for response generation.
    /// - Navigate back to the `AppState::SelectPrompt` state.
    /// - Edit scrolls.
    ///
    /// ### Arguments:
    /// `key_event` - The user input event.
    /// `pool` - The database connection pool.
    ///
    /// ### Returns:
    /// - `Result<AppState>`: Returns the next state of the application.
    async fn process_ask_model_input(
        &mut self,
        key_event: InputEvent,
        pool: &SqlitePool,
    ) -> Result<AppState> {
        match key_event {
            InputEvent::AskModel => {
                if self.user_config.is_some() && self.user_config.as_ref().unwrap().ask_conf {
                    // Require confirmation for specific models
                    return Ok(AppState::AskModelConfirmation);
                } else {
                    return self.produce_question(pool).await;
                }
            }
            InputEvent::SwitchBranch => Ok(AppState::SelectPrompt),
            InputEvent::EditScrolls => Ok(AppState::EditScrolls),
            InputEvent::ChangeProject => Ok(AppState::SelectProject),
            InputEvent::Quit => Ok(AppState::Quit),
            _ => Ok(AppState::AskModel),
        }
    }

    /// Processes user input in the `AppState::EditScrolls` state.
    ///
    /// In this state, users can:
    /// - Add new scrolls to a project.
    /// - Delete existing scrolls from a project.
    /// - Navigate back to prompt selection.
    /// - Change the active project.
    ///
    /// ### Arguments:
    /// `key_event` - The user input event.
    /// `pool` - The database connection pool.
    ///
    /// ### Returns:
    /// - `Result<AppState>`: Returns the next state of the application.
    async fn process_edit_scrolls_input(
        &mut self,
        key_event: InputEvent,
        pool: &SqlitePool,
    ) -> Result<AppState> {
        match key_event {
            InputEvent::New => {
                if let Some(project) = &self.current_project {
                    disable_raw_mode()?;
                    let selected_scroll = select_files(Some(&project.project_path))
                        .unwrap()
                        .unwrap_or(String::from(""));
                    enable_raw_mode()?;
                    // Fetch all scrolls from cache
                    let scrolls: Vec<Scroll> = if let Some(cache) = &self.scroll_list_cache {
                        cache.clone()
                    } else {
                        let s = get_scrolls(pool, &project.project_id).await?;
                        self.scroll_list_cache = Some(s.clone());
                        s
                    };
                    let old_scroll = scrolls.iter().find(|s| s.scroll_path == selected_scroll);
                    if old_scroll.is_none() {
                        let new_scroll = read_file(&selected_scroll, &project.project_id, None)?;
                        store_scroll(pool, &new_scroll).await?;
                        // Initial scroll
                        self.scroll_list_cache = Some(vec![new_scroll]);
                    }
                }
                return Ok(AppState::EditScrolls);
            }
            InputEvent::Delete => {
                if let Some(project) = &self.current_project {
                    // Fetch all scrolls from cache
                    let scrolls: Vec<Scroll> = if let Some(cache) = &self.scroll_list_cache {
                        cache.clone()
                    } else {
                        let s = get_scrolls(pool, &project.project_id).await?;
                        self.scroll_list_cache = Some(s.clone());
                        s
                    };
                    let scroll_names = scrolls
                        .iter()
                        .map(|s| s.scroll_path.clone())
                        .collect::<Vec<_>>();

                    disable_raw_mode()?;
                    if let Some(selected_scroll) = item_selector(scroll_names.clone())? {
                        enable_raw_mode()?;
                        let idx = scroll_names
                            .iter()
                            .position(|s| s == &selected_scroll)
                            .unwrap();

                        if idx < scrolls.len() {
                            delete_scroll(pool, &scrolls[idx].scroll_id).await?;
                            
                            // Clear the cache
                            self.scroll_list_cache = None;
                        }
                    } else {
                        enable_raw_mode()?;
                        return Ok(AppState::EditScrolls);
                    }
                }
                return Ok(AppState::EditScrolls);
            }
            InputEvent::SwitchBranch => Ok(AppState::SelectPrompt),
            InputEvent::ChangeProject => Ok(AppState::SelectProject),
            InputEvent::AskModel => Ok(AppState::AskModel),
            InputEvent::Quit => Ok(AppState::Quit),
            _ => Ok(AppState::EditScrolls),
        }
    }

    /// Processes user input in the `AppState::AskModelConfirmation` state.
    ///
    /// In this state, users:
    /// - Confirm or cancel their intent to query the AI for response generation.
    ///
    /// ### Arguments:
    /// `key_event` - The user input for confirmation or cancellation.
    /// `pool` - The database connection pool.
    ///
    /// ### Returns:
    /// - `Result<AppState>`: Returns the next state of the application.
    async fn process_confirmation_popup_input(
        &mut self,
        key_event: InputEvent,
        pool: &SqlitePool,
    ) -> Result<AppState> {
        match key_event {
            InputEvent::Confirm => {
                // User confirmed the action
                return self.produce_question(pool).await;
            }
            InputEvent::Cancel => {
                // User cancelled; return to the previous state (e.g., `AskModel`)
                return Ok(AppState::AskModel);
            }
            _ => {}
        }
        Ok(AppState::AskModelConfirmation)
    }

    /// Generates and sends a new question to the AI model for processing.
    ///
    /// This function:
    /// - Prepares the current prompt chain and associated scrolls.
    /// - Executes a query to the AI model.
    /// - Stores the resulting output as a new prompt.
    /// - Updates the user interface with the latest prompt chain.
    ///
    /// ### Arguments:
    /// `pool` - The database connection pool.
    ///
    /// ### Returns:
    /// - `Result<AppState>`: The next state of the application is determined (usually remains `AppState::AskModel`).
    async fn produce_question(&mut self, pool: &SqlitePool) -> Result<AppState> {
        if let Some(project) = &self.current_project {
            // Fetch all prompts from cache
            let scrolls: Vec<Scroll> = if let Some(cache) = &self.scroll_list_cache {
                cache.clone()
            } else {
                let s = get_scrolls(pool, &project.project_id).await?;
                self.scroll_list_cache = Some(s.clone());
                s
            };
            let mut new_scrolls = Vec::new();
            for scroll in scrolls.iter() {
                let new_scroll = update_scroll_content(pool, scroll).await?;
                new_scrolls.push(new_scroll);
            }
            let sys_prompt = system_prompt(&new_scrolls).await;

            let prompts = get_prompts(pool, &project.project_id).await?;

            let mut chain: Option<Vec<Prompt>> = None;
            if let Some(curr_prompt) = &self.current_prompt {
                chain = Some(prompt_chain(&prompts, curr_prompt));
            }

            let final_prompt = chain_match_canvas(project).unwrap_or(String::from("."));

            let prompt_chain: Option<Vec<AiPrompt>> = match chain {
                Some(prompts) => Some(
                    prompts
                        .iter()
                        .map(|p| AiPrompt {
                            content: p.content.to_owned(),
                            output: p.output.to_owned(),
                        })
                        .collect(),
                ),
                None => None,
            };

            let question = Question {
                system_prompt: if sys_prompt.is_empty() {
                    None
                } else {
                    Some(sys_prompt)
                },
                messages: prompt_chain,
                new_prompt: final_prompt.to_owned(),
            };

            let output =
                ask_question(&self.user_config.as_ref().unwrap().ai_conf, question).await?;

            let new_prompt = Prompt::new(
                &project.project_id,
                &final_prompt,
                &output,
                &self
                    .current_prompt
                    .as_ref()
                    .map_or(project.project_id.clone(), |p| p.prompt_id.clone()),
            );

            store_prompt(pool, &new_prompt).await?;
            self.current_prompt = Some(new_prompt);

            // Clear cache
            self.prompt_list_cache = None;

            let mut new_prompts = prompts.clone();
            new_prompts.push(self.current_prompt.as_ref().unwrap().clone());

            chain_into_canvas(project, Some(&new_prompts), self.current_prompt.as_ref())?;
        }
        Ok(AppState::AskModel)
    }
}
