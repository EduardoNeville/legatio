use sqlx::sqlite::SqlitePool;
use anyhow::Result;
use crate::utils::{
    logger::{log_error, log_info}, structs::{Prompt, Scroll}
};

/// Stores a prompt into the database.
pub async fn store_prompt(pool: &SqlitePool, prompt: &Prompt) -> Result<()> {
    log_info("Storing prompt in the database");

    if let Err(error) = sqlx::query(
        "INSERT INTO prompts (prompt_id, scroll_id, content, output, next_prompt_id) VALUES ($1, $2, $3, $4, $5)")
        .bind(&prompt.prompt_id)
        .bind(&prompt.scroll_id)
        .bind(&prompt.content)
        .bind(&prompt.output)
        .bind(&prompt.next_prompt_id)
        .execute(pool)
        .await
    {
        log_error(&format!("Failed to insert prompt: {}", error));
        return Err(error.into());
    }

    Ok(())
}

// Sorted from first to last prompt on the list
pub async fn get_prompts_from_scroll(pool: &SqlitePool, scroll: &Scroll)-> Result<Vec<Prompt>> {
    let prompts: Vec<Prompt> = sqlx::query_as::<_, Prompt>(
        "SELECT * 
        FROM prompts
        WHERE scroll_id = $1;")
        .bind(&scroll.scroll_id)
        .fetch_all(pool)
        .await
        .unwrap();

    println!("Current prompts: \n");
    for (idx, row) in prompts.iter().enumerate() {
        println!(" [{}]: curr_id {}, next_id {} \n", idx, row.prompt_id, row.next_prompt_id);
    }

    Ok(prompts)
}

pub async fn update_prompt(pool: &SqlitePool, prompt: &Prompt, answer: &str) -> Result<()>{

    sqlx::query(
        "UPDATE prompts 
        SET output = $1 
        WHERE prompt_id = $2")
        .bind(&answer)
        .bind(&prompt.prompt_id)
        .execute(pool)
        .await
        .unwrap();

    Ok(())

}
