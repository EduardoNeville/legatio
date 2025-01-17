use std::io;

use anyhow::Result;
use ratatui::{backend::CrosstermBackend, Terminal};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::style::{Style, Color};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::utils::structs::Project;

pub enum AppState {
    SelectProject,
    SelectPrompt,
    AskModel,
    EditScrolls,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InputEvent {
    Select,
    New,
    Delete,
    Quit,
    Edit,
    ChangeProject,
    ScrollMode,
    Invalid,
}

impl From<KeyEvent> for InputEvent {
    fn from(key_event: KeyEvent) -> InputEvent {
        match key_event.code {
            KeyCode::Char('s') => InputEvent::Select,
            KeyCode::Char('n') => InputEvent::New,
            KeyCode::Char('d') => InputEvent::Delete,
            KeyCode::Char('q') => InputEvent::Quit,
            KeyCode::Char('e') => InputEvent::Edit,
            KeyCode::Char('c') => InputEvent::ChangeProject,
            KeyCode::Char('l') => InputEvent::ScrollMode,
            _ => InputEvent::Invalid,
        }
    }
}

pub fn format_project_title(current_project: &Option<Project>) -> String {
    match current_project {
        Some(project) => format!(
            "[ Current Project: {} ]",
            project.project_path.split('/').last().unwrap_or("")
        ),
        None => "[ Projects ]".to_string(),
    }
}

pub fn build_project_list(projects: &[Project]) -> (Vec<ListItem>, Vec<String>) {
    let mut items = Vec::new();
    let mut proj_items = Vec::new();
    for project in projects {
        let proj_name = format!(" -[ {:?} ]-", project.project_path.split('/').last().unwrap_or(""));
        proj_items.push(proj_name.clone());
        items.push(ListItem::new(proj_name));
    }
    items.push(ListItem::new("[s] Select Project"));
    items.push(ListItem::new("[n] New Project"));
    items.push(ListItem::new("[d] Delete Project"));
    items.push(ListItem::new("[q] Quit"));
    (items, proj_items)
}

pub fn display_project_list(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    title: &str,
    items: Vec<ListItem>,
) -> Result<()> {
    terminal.draw(|frame| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(frame.area());
        let block = Block::default().title(title).borders(Borders::ALL);
        let list = List::new(items).block(block);
        frame.render_widget(list, chunks[0]);
    })?;
    Ok(())
}
