use dotenv::dotenv;
use services::legatio::Legatio;
use utils::db_utils::get_db_pool;
//use db::app_state::{self, get_app_state};

mod services;
mod utils;
mod db;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    
    // Initialisers
    dotenv().ok();
    env_logger::init();
    let db_url = "sqlite://legatio.db";
    let pool = get_db_pool(&db_url).await.unwrap();
    let mut app = Legatio::new();
    app.run(&pool).await.unwrap();

    Ok(())
    //let terminal = ratatui::init();
    //let result = AppState::new().run(&pool, terminal).await;
    //ratatui::restore();
    //return result;
    
}

