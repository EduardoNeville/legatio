use sqlx::sqlite::SqlitePool;
use anyhow::{Ok, Result};
use crate::utils::{
    db_utils::delete_module,
    logger::log_error,
    structs::Prompt
};

/// Stores a prompt into the database.
pub async fn store_prompt(pool: &SqlitePool, prompt: &Prompt) -> Result<()> {
    // Use RETURNING clause (if supported by SQLite) to fetch the `prompt_id`.
    if let Err(error) = sqlx::query(
        "INSERT INTO prompts (prompt_id, project_id, prev_prompt_id, content, output) 
         VALUES ($1, $2, $3, $4, $5)")
    .bind(&prompt.prompt_id)
    .bind(&prompt.project_id)
    .bind(&prompt.prev_prompt_id)
    .bind(&prompt.content)
    .bind(&prompt.output)
    .execute(pool)
    .await 
    {
        log_error(&format!("FAILED :: INSERT prompt_id: [{}]", 
            prompt.prompt_id,
        ));
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

pub async fn update_prompt(
    pool: &SqlitePool,
    col_set_name: &str,
    new_value: &str,
    col_comp_name: &str,
    col_comp_val: &str
) -> Result<()>{
    let query = format!("UPDATE prompts SET {} = ? WHERE {} = ?",
        &col_set_name,
        &col_comp_name,
    );

    if let Err(error) = sqlx::query(&query)
        .bind(&new_value)
        .bind(&col_comp_val)
        .execute(pool)
        .await
    {
        log_error(&format!(
            "FAILED :: UPDATE prompts SET {} = {} WHERE {} = {}",
            &col_set_name,
            &new_value,
            &col_comp_name,
            &col_comp_val
        ));
        return Err(error.into());
    }
    Ok(())
}

pub async fn delete_prompt(
    pool: &SqlitePool,
    prompt: &Prompt
) -> Result<()> {
    if let Err(error) = delete_module(pool, &"prompts", &"prompt_id", &prompt.prompt_id)
        .await
    {
        log_error(&format!("FAILED :: DELETE prompt_id: [{}]", 
            prompt.prompt_id,
        ));
        return Err(error.into());
    }

    if let Err(error) = update_prompt(
        pool,
        &"prev_prompt_id",
        &prompt.prev_prompt_id,
        &"prev_prompt_id",
        &prompt.prompt_id)
        .await
    {
        log_error(&format!("FAILED :: DELETE -> UPDATE prompt_id: [{}]", 
            prompt.prompt_id,
        ));
        return Err(error.into());
    }

    Ok(())

}
