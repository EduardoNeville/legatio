use services::legatio::Legatio;
use utils::{db_utils::get_db_pool, logger::initialize_logger};

mod core;
mod services;
mod utils;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    initialize_logger("app.log")?;
    let pool = get_db_pool().await?;
    let mut app = Legatio::new();
    app.run(&pool).await?;

    Ok(())
}
