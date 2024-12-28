use dotenv::dotenv;
use services::flow::flow;
use utils::db_utils::get_db_pool;
use std::{env};

mod services;
mod utils;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let api_key = env::var("OPENAI_API_KEY")?;
    let db_url = String::from("sqlite://legatio.db");
    let pool = get_db_pool(&db_url).await?;

    flow(&pool).await;

    //
    //let ans = get_openai_response(&api_key, &system_prompt, &user_prompt).await?;

    //println!("GPT Answer:\n{}", ans);

    Ok(())
}
