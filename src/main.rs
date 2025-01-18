use services::legatio::Legatio;
use utils::{db_utils::get_db_pool, logger::initialize_logger};
//use db::app_state::{self, get_app_state};

mod services;
mod utils;
mod db;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {

    initialize_logger("app.log")?;

    let db_url = "sqlite://legatio.db";
    let pool = get_db_pool(&db_url).await.unwrap();
    let mut app = Legatio::new();
    app.run(&pool).await.unwrap();

    Ok(())
    
}

