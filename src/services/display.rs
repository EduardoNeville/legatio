use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
                code: KeyCode::Char('p'),
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
