use ratatui::text::Line;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::any::type_name;

use crate::utils::structs::{Project, Prompt};

#[derive(Clone, Copy)]
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
    SwitchBranch,
    ChangeProject,
    EditScrolls,
    AskModel,
    Quit,
    NoOp,
}

impl From<KeyEvent> for InputEvent {
    fn from(event: KeyEvent) -> Self {
        match event {
            KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::NONE,
                kind: _,
                state: _,
            } => InputEvent::Select,
            KeyEvent {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::NONE,
                kind: _,
                state: _,
            } => InputEvent::New,
            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::NONE,
                kind: _,
                state: _,
            } => InputEvent::Delete,
            KeyEvent {
                code: KeyCode::Char('b'),
                modifiers: KeyModifiers::NONE,
                kind: _,
                state: _,
            } => InputEvent::SwitchBranch,
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::NONE,
                kind: _,
                state: _,
            } => InputEvent::ChangeProject,
            KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::NONE,
                kind: _,
                state: _,
            } => InputEvent::EditScrolls,
            KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::NONE,
                kind: _,
                state: _,
            } => InputEvent::AskModel,
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
                kind: _,
                state: _,
            } => InputEvent::Quit,
            _ => InputEvent::NoOp,
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

pub fn build_select_project(projects: &[Project])-> (Vec<Line<'static>>, Vec<String>) {
    let mut proj_items: Vec<Line> = vec![];
    let mut str_items: Vec<String> = vec![];
    for project in projects.iter() {
        let proj_name = format!(" -[ {:?} ]-", project.project_path.split('/').last().unwrap_or(""));
        str_items.push(proj_name.to_owned());
        proj_items.push(Line::from(proj_name));
    }
    return (proj_items, str_items)
}

