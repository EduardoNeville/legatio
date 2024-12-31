use sqlx::sqlite::SqlitePool;
use anyhow::Result;
use crate::utils::structs::{Prompt, Scroll};
use crate::utils::logger::{log_info, log_error};

/// Stores a scroll into the database.
pub async fn store_scroll(pool: &SqlitePool, scroll: &Scroll) -> Result<()> {
    log_info("Storing scroll in the database");

    if let Err(error) = sqlx::query("INSERT INTO scrolls (scroll_id, project_id, init_prompt_id) VALUES ($1, $2, $3)")
        .bind(&scroll.scroll_id)
        .bind(&scroll.project_id)
        .bind(&scroll.init_prompt_id)
        .execute(pool)
        .await
    {
        log_error(&format!("Failed to store scroll: {}", error));
        return Err(error.into());
    }

    Ok(())
}

/// Retrieves scrolls for a specific project.
pub async fn get_scrolls(pool: &SqlitePool, project_id: &String) -> Result<Vec<Scroll>> {
    log_info("Fetching scrolls for a project");

    let results = sqlx::query_as::<_, Scroll>(
        "SELECT * FROM scrolls WHERE project_id = $1;")
        .bind(project_id)
        .fetch_all(pool)
        .await;

    match results {
        Ok(scrolls) => Ok(scrolls),
        Err(error) => {
            log_error(&format!("Failed to fetch scrolls: {}", error));
            Err(error.into())
        }
    }
}

pub async fn update_scroll(pool: &SqlitePool, scroll: &Scroll, prompt: &Prompt) -> Result<()> {
    if scroll.init_prompt_id == "" {
        sqlx::query(
            "UPDATE scrolls 
            SET init_prompt_id = $1 
            WHERE scroll_id = $2")
            .bind(&prompt.prompt_id)
            .bind(&scroll.scroll_id)
            .execute(pool)
            .await
            .unwrap();
    }

    let prompts: Vec<Prompt> = sqlx::query_as::<_, Prompt>(
        "SELECT * 
        FROM prompts 
        WHERE scroll_id = $1;") // Find last prompt
        .bind(&prompt.scroll_id)
        .fetch_all(pool)
        .await
        .unwrap();

    for (idx, row) in prompts.iter().enumerate() {
        println!(" [{}]: content {}, curr_id {}, next_id {} \n", idx, row.content, row.prompt_id, row.next_prompt_id);
    }

    if let Some(last_prompt) = prompts.iter().find(|p| p.next_prompt_id == prompt.next_prompt_id) {
        sqlx::query(
            "UPDATE prompts 
            SET next_prompt_id = $1 
            WHERE prompt_id = $2")
            .bind(&prompt.prompt_id)
            .bind(&last_prompt.prompt_id)
            .execute(pool)
            .await
            .unwrap();

    } else {
        println!("No matching prompt found.");
    }

    Ok(())
}
