use dotenv::dotenv;
use services::flow::flow;
use ui::app::AppState;
use utils::db_utils::get_db_pool;
use db::app_state::{self, get_app_state};

mod services;
mod utils;
mod db;
mod ui;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialisers
    dotenv().ok();
    env_logger::init();
    let db_url = "sqlite://legatio.db";
    let pool = get_db_pool(&db_url).await.unwrap();
    let terminal = ratatui::init();
    let result = AppState::new().run(&pool, terminal).await;
    ratatui::restore();
    return result;
    
    //let _ = flow(&pool).await;
}

