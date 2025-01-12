use sqlx::sqlite::SqlitePool;
use anyhow::Result;
use crate::utils::{
    logger::{log_error, log_info}, structs::Prompt
};

/// Stores a prompt into the database.
pub async fn store_prompt(pool: &SqlitePool, prompt: &Prompt) -> Result<()> {
    log_info("Storing prompt in the database");

    if let Err(error) = sqlx::query(
        "INSERT INTO prompts (prompt_id, project_id, content, output, prev_prompt_id, idx) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(&prompt.prompt_id)
        .bind(&prompt.project_id)
        .bind(&prompt.content)
        .bind(&prompt.output)
        .bind(&prompt.prev_prompt_id)
        .bind(&prompt.idx)
        .execute(pool)
        .await
    {
        log_error(&format!("Failed to insert prompt: {}", error));
        return Err(error.into());
    }

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
