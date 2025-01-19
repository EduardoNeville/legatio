use sqlx::sqlite::SqlitePool;
use anyhow::{Ok, Result};
use std::collections::HashMap;
use crate::utils::{
    db_utils::delete_module,
    logger::log_error,
    structs::{Scroll, Prompt}
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

pub fn system_prompt(scrolls: &[Scroll])-> String {
    let system_prompt = scrolls.iter()
        .map(|scroll| {
            let scroll_name = scroll.scroll_path.rsplit('/').next().unwrap_or(""); // Handles empty paths safely
            format!("```{:?}\n{:?}```\n", scroll_name, scroll.content)
        })
        .collect::<Vec<_>>()
        .join(""); // Joining avoids intermediate allocations with push_str
    
    return system_prompt
}

pub fn prompt_chain(prompts: &[Prompt], prompt: &Prompt) -> Vec<Prompt> {
    let mut prompt_map: HashMap<&str, &Prompt> = prompts
        .into_iter()
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

    return chain;
}

pub fn format_prompt(p: &Prompt)-> (String, String) {
    let p_str = format!(" |- Prompt: {:?} ",
        if p.content.chars().count() < 40 {
            p.content.replace('\n', " ").to_string()
        } else {
            p.content[0..40].replace('\n', " ").to_string()
        },
    );
    let o_str = format!(" |- Output: {:?}",
        if p.output.chars().count() < 40 {
            p.output.replace('\n', " ").to_string()
        } else {
            p.output[0..40].replace('\n', " ").to_string()
        }
    );

    (p_str, o_str)
}

pub fn format_prompt_depth(p: &Prompt, b_depth: &str)-> (String, String) {
    let p_str = format!("{b_depth}> Prompt: {:?} ",
        if p.content.chars().count() < 40 {
            p.content.replace('\n', " ").to_string()
        } else {
            p.content[0..40].replace('\n', " ").to_string()
        },
    );
    let o_str = format!("{b_depth}> Output: {:?}",
        if p.output.chars().count() < 40 {
            p.output.replace('\n', " ").to_string()
        } else {
            p.output[0..40].replace('\n', " ").to_string()
        }
    );

    (p_str, o_str)
}
