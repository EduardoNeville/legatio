use crate::utils::{
    db_utils::delete_module,
    error::AppError,
    logger::log_error,
    structs::{Prompt, Scroll},
};
use anyhow::{Ok, Result};
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;

/// Stores a prompt into the database.
pub async fn store_prompt(pool: &SqlitePool, prompt: &Prompt) -> Result<()> {
    sqlx::query(
        "INSERT INTO prompts (prompt_id, project_id, prev_prompt_id, content, output) 
         SELECT $1, $2, $3, $4, $5
         WHERE NOT EXISTS (
             SELECT 1 FROM prompts WHERE content = $4 AND output = $5
         )",
    )
    .bind(&prompt.prompt_id) // Values to insert
    .bind(&prompt.project_id)
    .bind(&prompt.prev_prompt_id)
    .bind(&prompt.content)
    .bind(&prompt.output)
    .execute(pool)
    .await
    .map_err(|err| {
        log_error(&format!(
            "FAILED :: INSERT prompt_id = {}, error: {}",
            prompt.prompt_id, err
        ));
        AppError::DatabaseError(format!(
            "Failed to store prompt with ID {}. Reason: {}",
            prompt.prompt_id, err
        ))
    })?;

    Ok(())
}

// Sorted from first to last prompt on the list
pub async fn get_prompts(pool: &SqlitePool, project_id: &str) -> Result<Vec<Prompt>> {
    let prompts = sqlx::query_as::<_, Prompt>(
        "SELECT * 
        FROM prompts
        WHERE project_id = $1;",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(|err| {
        log_error(&format!(
            "Failed to get prompts for project_id {}. Reason: {}",
            project_id.to_owned(),
            err
        ));
        AppError::DatabaseError(format!(
            "Failed to get prompts for project_id {}. Reason: {}",
            project_id.to_owned(),
            err
        ))
    })?;

    Ok(prompts)
}

pub async fn update_prompt(
    pool: &SqlitePool,
    col_set_name: &str,
    new_value: &str,
    col_comp_name: &str,
    col_comp_val: &str,
) -> Result<()> {
    let query = format!(
        "UPDATE prompts SET {} = ? WHERE {} = ?",
        &col_set_name, &col_comp_name,
    );

    sqlx::query(&query)
        .bind(new_value)
        .bind(col_comp_val)
        .execute(pool)
        .await
        .map_err(|err| {
            log_error(&format!(
                "FAILED :: UPDATE prompts SET {} = {} WHERE {} = {}",
                &col_set_name, &new_value, &col_comp_name, &col_comp_val
            ));
            AppError::DatabaseError(format!(
                "FAILED :: UPDATE prompts SET {} = {} WHERE {} = {} \n {}",
                &col_set_name, &new_value, &col_comp_name, &col_comp_val, err
            ))
        })?;
    Ok(())
}

pub async fn delete_prompt(pool: &SqlitePool, prompt: &Prompt) -> Result<()> {
    if let Err(error) = delete_module(pool, "prompts", "prompt_id", &prompt.prompt_id).await {
        log_error(&format!(
            "FAILED :: DELETE prompt_id: [{}]",
            prompt.prompt_id,
        ));
        return Err(error.into());
    }

    if let Err(error) = update_prompt(
        pool,
        "prev_prompt_id",
        &prompt.prev_prompt_id,
        "prev_prompt_id",
        &prompt.prompt_id,
    )
    .await
    {
        log_error(&format!(
            "FAILED :: DELETE -> UPDATE prompt_id: [{}]",
            prompt.prompt_id,
        ));
        return Err(error);
    }

    Ok(())
}

pub async fn system_prompt(scrolls: &[Scroll]) -> String {
    let mut system_prompt = String::new();

    for scroll in scrolls.iter() {
        let scroll_name = scroll.scroll_path.rsplit('/').next().unwrap_or(""); // Handles empty paths safely

        system_prompt.push_str(&format!("```{}\n{}```\n", scroll_name, scroll.content));
    }

    system_prompt
}

pub fn prompt_chain(prompts: &[Prompt], prompt: &Prompt) -> Vec<Prompt> {
    let mut prompt_map: HashMap<&str, &Prompt> = prompts
        .iter()
        .map(|prompt| (prompt.prompt_id.as_ref(), prompt))
        .collect();

    let mut chain = Vec::<Prompt>::new();
    let mut current_id: Option<&str> = Some(prompt.prompt_id.as_ref());

    while let Some(id) = current_id {
        if let Some(prompt) = prompt_map.remove(id) {
            current_id = if prompt.prev_prompt_id.is_empty() {
                None
            } else {
                Some(prompt.prev_prompt_id.as_ref())
            };
            chain.push(prompt.to_owned());
        } else {
            break;
        }
    }

    chain
}

pub fn format_prompt(p: &Prompt) -> (String, String) {
    let p_str = format!(" |- Prompt: {}", p.content.replace('\n', " ").to_string());

    let o_str = format!(" |  Output: {}", p.output.replace('\n', " ").to_string());

    (p_str, o_str)
}

pub fn format_prompt_depth(p: &Prompt, b_depth: &str) -> (String, String) {
    let p_str = format!(
        "{b_depth}> Prompt: {}",
        p.content.replace('\n', " ").to_string()
    );

    let o_str = format!(
        "{b_depth}> Output: {}",
        p.output.replace('\n', " ").to_string()
    );

    (p_str, o_str)
}
