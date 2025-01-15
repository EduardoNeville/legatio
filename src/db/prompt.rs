use sqlx::sqlite::SqlitePool;
use anyhow::{Ok, Result};
use crate::utils::{logger::log_info, structs::Prompt};

/// Stores a prompt into the database.
pub async fn store_prompt(pool: &SqlitePool, prompt: &mut Prompt) -> Result<()> {
    log_info("Storing prompt in the database");

    // Use RETURNING clause (if supported by SQLite) to fetch the `prompt_id`.
    let _ = sqlx::query(
        "INSERT INTO prompts (prompt_id, project_id, prev_prompt_id, content, output) 
         VALUES ($1, $2, $3, $4, $5)
         RETURNING idx"
    )
    .bind(&prompt.prompt_id)
    .bind(&prompt.project_id)
    .bind(&prompt.prev_prompt_id)
    .bind(&prompt.content)
    .bind(&prompt.output)
    .fetch_one(pool)
    .await;

    Ok(())
}

// Sorted from first to last prompt on the list
pub async fn get_prompts(pool: &SqlitePool, project_id: &str)-> Result<Vec<Prompt>> {
    let prompts: Vec<Prompt> = sqlx::query_as::<_, Prompt>(
        "SELECT * 
        FROM prompts
        WHERE project_id = $1;")
        .bind(project_id)
        .fetch_all(pool)
        .await
        .unwrap();

    Ok(prompts)
}

pub async fn update_prompt(pool: &SqlitePool, prompt: &Prompt, column: &str, new_value: &str) -> Result<()>{
    sqlx::query(
        "UPDATE prompts 
        SET $1 = $2 
        WHERE prompt_id = $3")
        .bind(&column)
        .bind(&new_value)
        .bind(&prompt.prompt_id)
        .execute(pool)
        .await
        .unwrap();
    Ok(())
}
