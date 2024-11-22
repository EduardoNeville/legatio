use dotenv::dotenv;
use std::{env, error::Error, io};
use tokio::sync::mpsc;

use crossterm::event::{self, Event};
use ratatui::{text::Text, Frame};

use sqlx::sqlite::SqlitePool;

mod services;
mod utils;

use anyhow::Result;

use utils::db_utils::{get_db_pool, store_files, store_prompt};
use utils::file_utils::{get_all_files_in_directory, read_files};
use utils::prompt_utils::construct_system_prompt;

use services::model::get_openai_response;
use services::search::select_files;

use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let api_key = env::var("OPENAI_API_KEY")?;

    let pool = get_db_pool().await?;

    // Get all files in the current directory (or specify directory as needed)
    let file_list = get_all_files_in_directory(".")?;
    let selected_files = select_files(&file_list)?;

    let files = read_files(&selected_files)?;

    store_files(&pool, &files).await?;

    let system_prompt = construct_system_prompt(&files);

    // Initialize terminal
    let mut terminal = ratatui::init();
        loop {
            terminal.draw(draw).expect("failed to draw frame");
            if matches!(event::read().expect("failed to read event"), Event::Key(_)) {
                break;
            }
        }
        ratatui::restore();
}

fn draw(frame: &mut Frame) {
    let text = Text::raw("Hello World!");
    frame.render_widget(text, frame.area());
}
