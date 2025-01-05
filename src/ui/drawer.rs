use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;
use sqlx::{Pool, Sqlite};
use crate::db::file::get_files;
use crate::db::scroll::get_scrolls;
use crate::ui::app::AppState;

// Helper data to pass into the rendering function after async operations
pub struct DrawerData {
    pub files: Vec<String>,
    pub scrolls: Vec<(String, bool)>, // (scroll_id, is_highlighted)
}

// Fetch data from the database asynchronously
pub async fn fetch_drawer_data(
    app_state: &mut AppState,
    pool: &Pool<Sqlite>,
) -> DrawerData {
    // Fetch files asynchronously
    let files = get_files(pool, &app_state.project_id.clone().unwrap())
        .await
        .unwrap_or_default()
        .iter()
        .map(|file| file.file_path.split("/").last().unwrap().to_string())
        .collect();

    // Fetch scrolls asynchronously
    let scrolls = get_scrolls(pool, &app_state.project_id.clone().unwrap())
        .await
        .unwrap_or_default()
        .iter()
        .map(|scroll| {
            let is_highlighted = Some(scroll.scroll_id.clone()) == app_state.display_item_id;
            (scroll.scroll_id.clone(), is_highlighted)
        })
        .collect();

    DrawerData { files, scrolls }
}

// Render the drawer synchronously using fetched data and the Frame
pub fn render_drawer(
    f: &mut Frame,
    chunk: Rect,
    data: &DrawerData,
) {
    let drawer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(50), // Files box
                Constraint::Percentage(50), // Scrolls box
            ]
            .as_ref(),
        )
        .split(chunk);

    // Render Files Box
    let file_items: Vec<ListItem> = data.files.iter()
        .map(|file| ListItem::new(file.clone()))
        .collect();
    let files_list = List::new(file_items)
        .block(Block::default().borders(Borders::ALL).title("Files"));
    f.render_widget(files_list, drawer_chunks[0]);

    // Render Scrolls Box
    let scroll_items: Vec<ListItem> = data.scrolls.iter()
        .map(|(scroll_id, is_highlighted)| {
            let style = if *is_highlighted {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(scroll_id.clone()).style(style)
        })
        .collect();
    let scrolls_list = List::new(scroll_items)
        .block(Block::default().borders(Borders::ALL).title("Scrolls"));
    f.render_widget(scrolls_list, drawer_chunks[1]);
}
