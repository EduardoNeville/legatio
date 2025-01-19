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
            format!("```{}\n{}```\n", scroll_name, scroll.content)
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
    let p_str = format!(" |- Prompt: {} ",
        if p.content.chars().count() < 40 {
            p.content.replace('\n', " ").to_string()
        } else {
            p.content[0..40].replace('\n', " ").to_string()
        },
    );
    let o_str = format!(" |- Output: {}",
        if p.output.chars().count() < 40 {
            p.output.replace('\n', " ").to_string()
        } else {
            p.output[0..40].replace('\n', " ").to_string()
        }
    );

    (p_str, o_str)
}

pub fn format_prompt_depth(p: &Prompt, b_depth: &str)-> (String, String) {
    let p_str = format!("{b_depth}> Prompt: {} ",
        if p.content.chars().count() < 40 {
            p.content.replace('\n', " ").to_string()
        } else {
            p.content[0..40].replace('\n', " ").to_string()
        },
    );
    let o_str = format!("{b_depth}> Output: {}",
        if p.output.chars().count() < 40 {
            p.output.replace('\n', " ").to_string()
        } else {
            p.output[0..40].replace('\n', " ").to_string()
        }
    );

    (p_str, o_str)
}

// &&&&&
// Tests
// &&&&&

#[cfg(test)]
mod tests {
    use super::*; // Import functions from the `prompts.rs` file
    use sqlx::sqlite::SqlitePoolOptions; // Required for testing functions involving the database
    use crate::utils::structs::{Prompt, Scroll};

    async fn create_test_pool() -> SqlitePool {
        // Create a temporary in-memory SQLite database for testing
        SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create database connection pool")
    }

    #[tokio::test]
    async fn test_store_prompt() {
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE prompts (
                prompt_id TEXT PRIMARY KEY,
                project_id TEXT,
                prev_prompt_id TEXT,
                content TEXT,
                output TEXT
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        let prompt = Prompt {
            prompt_id: "test_prompt".to_string(),
            project_id: "test_project".to_string(),
            prev_prompt_id: "prev_test_prompt".to_string(),
            content: "Test Prompt Content".to_string(),
            output: "Test Prompt Output".to_string(),
        };

        let result = store_prompt(&pool, &prompt).await;
        assert!(result.is_ok());

        // Verify that the prompt was stored in the database
        let stored_prompt: (String, String, String, String, String) = sqlx::query_as(
            "SELECT prompt_id, project_id, prev_prompt_id, content, output FROM prompts"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(stored_prompt.0, "test_prompt");
        assert_eq!(stored_prompt.1, "test_project");
        assert_eq!(stored_prompt.2, "prev_test_prompt");
        assert_eq!(stored_prompt.3, "Test Prompt Content");
        assert_eq!(stored_prompt.4, "Test Prompt Output");
    }

    #[tokio::test]
    async fn test_get_prompts() {
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE prompts (
                prompt_id TEXT PRIMARY KEY,
                project_id TEXT,
                prev_prompt_id TEXT,
                content TEXT,
                output TEXT
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO prompts (prompt_id, project_id, prev_prompt_id, content, output) 
            VALUES ('test_prompt', 'test_project', 'prev_test_prompt', 'Content', 'Output')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let prompts = get_prompts(&pool, "test_project").await.unwrap();
        assert_eq!(prompts.len(), 1);

        let prompt = &prompts[0];
        assert_eq!(prompt.prompt_id, "test_prompt");
        assert_eq!(prompt.project_id, "test_project");
        assert_eq!(prompt.prev_prompt_id, "prev_test_prompt");
        assert_eq!(prompt.content, "Content");
        assert_eq!(prompt.output, "Output");
    }

    #[tokio::test]
    async fn test_update_prompt() {
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE prompts (
                prompt_id TEXT PRIMARY KEY,
                project_id TEXT,
                prev_prompt_id TEXT,
                content TEXT,
                output TEXT
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO prompts (prompt_id, project_id, prev_prompt_id, content, output) 
            VALUES ('test_prompt', 'test_project', 'prev_test_prompt', 'Content', 'Output')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let result = update_prompt(&pool, "content", "Updated Content", "prompt_id", "test_prompt").await;
        assert!(result.is_ok());

        let updated_prompt: String = sqlx::query_scalar(
            "SELECT content FROM prompts WHERE prompt_id = 'test_prompt'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(updated_prompt, "Updated Content");
    }

    #[tokio::test]
    async fn test_delete_prompt() {
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE prompts (
                prompt_id TEXT PRIMARY KEY,
                project_id TEXT,
                prev_prompt_id TEXT,
                content TEXT,
                output TEXT
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        let prompt = Prompt {
            prompt_id: "test_prompt".to_string(),
            project_id: "test_project".to_string(),
            prev_prompt_id: "prev_test_prompt".to_string(),
            content: "Content".to_string(),
            output: "Output".to_string(),
        };

        store_prompt(&pool, &prompt).await.unwrap();

        let result = delete_prompt(&pool, &prompt).await;
        assert!(result.is_ok());

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompts WHERE prompt_id = 'test_prompt'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_system_prompt() {
        let scrolls = vec![
            Scroll {
                project_id: "project-1".to_string(),
                scroll_id: "scroll-1".to_string(),
                scroll_path: "/path/to/scroll_one".to_string(),
                content: "Scroll One Content".to_string(),
            },
            Scroll {
                project_id: "project-2".to_string(),
                scroll_id: "scroll-2".to_string(),
                scroll_path: "/path/to/scroll_two".to_string(),
                content: "Scroll Two Content".to_string(),
            },
        ];

        let result = system_prompt(&scrolls);

        assert!(result.contains("```"));
        assert!(result.contains("scroll_one"));
        assert!(result.contains("Scroll One Content"));
        assert!(result.contains("scroll_two"));
        assert!(result.contains("Scroll Two Content"));
    }
}
