use ratatui::{
    widgets::{Block, Borders, Paragraph},
    layout::Rect,
    Frame,
};

use crate::ui::app::AppState;

pub fn render_master_box(f: &mut Frame, chunk: Rect, app_state: &AppState) {
    if let Some(item_id) = &app_state.display_item_id {
        let item = format!("Displaying item_id: {}", item_id); // Placeholder for display_item logic
        let paragraph = Paragraph::new(item)
            .block(Block::default().borders(Borders::ALL).title("Main Box"));
        f.render_widget(paragraph, chunk);
    } else {
        // Input mode
        let input_text = if app_state.input_mode {
            "Write your input here..."
        } else {
            "Press Ctrl-Enter to run ask_ctrl"
        };
        let paragraph = Paragraph::new(input_text)
            .block(Block::default().borders(Borders::ALL).title("Input Mode"));
        f.render_widget(paragraph, chunk);
    }
}
