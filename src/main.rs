use dotenv::dotenv;
use services::flow::flow;
use utils::db_utils::get_db_pool;

mod services;
mod utils;
mod db;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let db_url = "sqlite://legatio.db";
    let pool = get_db_pool(&db_url).await?;
    let _ = flow(&pool).await;

    Ok(())
}
