use ratatui::{
    style::Style,
    widgets::{Block, Borders, List, ListItem},
    layout::Rect,
    Frame,
};

use crate::{ui::app::AppState, utils::structs::Prompt};

pub fn render_prompts_box(app_state: &AppState, f: &mut Frame, chunk: Rect, prompts: Vec<Prompt> ) {
    let prompt_items: Vec<ListItem> = prompts
        .iter()
        .enumerate()
        .map(|(index, prompt)| {
            let style = if Some(index) == app_state.highlighted_prompt_index {
                Style::default().fg(ratatui::style::Color::Yellow)
            } else {
                Style::default()
            };
            ListItem::new(format!("Prompt: {}\nAnswer: {}", prompt.content, prompt.output)).style(style)
        })
        .collect();
    let prompts_list = List::new(prompt_items)
        .block(Block::default().borders(Borders::ALL).title("Prompts"));
    f.render_widget(prompts_list, chunk);
}
