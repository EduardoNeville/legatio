use sqlx::sqlite::SqlitePool;
use futures::future;
use anyhow::Result;
use crate::utils::db_utils::delete_module;
use crate::utils::structs::Scroll;
use crate::utils::logger::{log_info, log_error};
use std::fs;

/// Inserts a scroll into the database.
pub async fn store_scroll(pool: &SqlitePool, scroll: &Scroll) -> Result<()> {
    if let Err(error) = sqlx::query(
        "INSERT INTO scrolls (scroll_id, scroll_path, content, project_id) VALUES ($1, $2, $3, $4)")
        .bind(&scroll.scroll_id)
        .bind(&scroll.scroll_path)
        .bind(&scroll.content)
        .bind(&scroll.project_id)
        .execute(pool)
        .await
    {
        log_error(&format!("FAILED :: INSERT scroll_id: [{}]", 
            scroll.scroll_id,
        ));
        return Err(error.into());
    }

    Ok(())
}

/// Inserts multiple scrolls into the database in parallel.
pub async fn store_scrolls(pool: &SqlitePool, scrolls: &[Scroll]) -> Result<()> {
    log_info("Attempting to store multiple scrolls");

    let results = future::join_all(scrolls.iter().map(|scroll| {
        let pool = pool.clone();
        async move {
            if let Err(error) = store_scroll(&pool, scroll).await {
                log_error(&format!("Failed to store scroll: {}", error));
                Err(error)
            } else {
                Ok(())
            }
        }
    }))
    .await;

    for result in results {
        if let Err(error) = result {
            return Err(error);
        }
    }

    log_info("All scrolls stored successfully");
    Ok(())
}

pub async fn get_scrolls(pool: &SqlitePool, project_id: &str)-> Result<Vec<Scroll>> {
    let scrolls_result: Vec<Scroll> = sqlx::query_as::<_, Scroll>(
        "SELECT *
        FROM scrolls 
        WHERE project_id = $1;")
        .bind(project_id)
        .fetch_all(pool)
        .await
        .unwrap();

    Ok(scrolls_result)
}

pub async fn delete_scroll(pool: &SqlitePool, scroll_id: &str) -> Result<()> {
    delete_module(pool, &"scrolls", &"scroll_id", scroll_id)
        .await
        .expect("Error in scroll deletion");

    Ok(())
}

pub fn read_file(file_path: &str, project_id: &str) -> Result<Scroll> {
    let content = fs::read_to_string(file_path)?;
    Ok(Scroll::new(file_path, &content, project_id))
}

#[cfg(test)]
mod tests {
    use super::*; // Import functions from the `scroll.rs` file for testing
    use sqlx::sqlite::SqlitePoolOptions;
    use std::fs;
    use crate::utils::structs::Scroll;

    async fn create_test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create database connection pool")
    }

    #[tokio::test]
    async fn test_store_scroll() {
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE scrolls (
                scroll_id TEXT PRIMARY KEY,
                scroll_path TEXT,
                content TEXT,
                project_id TEXT
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        let scroll = Scroll {
            scroll_id: "scroll_1".to_string(),
            scroll_path: "/path/to/scroll_1".to_string(),
            content: "Test Scroll Content".to_string(),
            project_id: "project_1".to_string(),
        };

        let result = store_scroll(&pool, &scroll).await;
        assert!(result.is_ok());

        // Verify that the scroll was stored
        let stored_scrolls = get_scrolls(&pool, "project_1").await.unwrap();
        assert_eq!(stored_scrolls.len(), 1);

        let stored_scroll = &stored_scrolls[0];
        assert_eq!(stored_scroll.scroll_id, "scroll_1");
        assert_eq!(stored_scroll.scroll_path, "/path/to/scroll_1");
        assert_eq!(stored_scroll.content, "Test Scroll Content");
        assert_eq!(stored_scroll.project_id, "project_1");
    }

    #[tokio::test]
    async fn test_get_scrolls() {
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE scrolls (
                scroll_id TEXT PRIMARY KEY,
                scroll_path TEXT,
                content TEXT,
                project_id TEXT
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO scrolls (scroll_id, scroll_path, content, project_id) 
             VALUES ('scroll_1', '/path/1', 'Content 1', 'project_1'),
                    ('scroll_2', '/path/2', 'Content 2', 'project_1');",
        )
        .execute(&pool)
        .await
        .unwrap();

        let scrolls = get_scrolls(&pool, "project_1").await.unwrap();
        assert_eq!(scrolls.len(), 2);

        assert_eq!(scrolls[0].scroll_id, "scroll_1");
        assert_eq!(scrolls[1].scroll_id, "scroll_2");
    }

    #[tokio::test]
    async fn test_delete_scroll() {
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE scrolls (
                scroll_id TEXT PRIMARY KEY,
                scroll_path TEXT,
                content TEXT,
                project_id TEXT
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO scrolls (scroll_id, scroll_path, content, project_id) 
             VALUES ('scroll_1', '/path/1', 'Content 1', 'project_1');",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Delete the scroll
        let delete_result = delete_scroll(&pool, "scroll_1").await;
        assert!(delete_result.is_ok());

        let scrolls = get_scrolls(&pool, "project_1").await.unwrap();
        assert!(scrolls.is_empty());
    }

    #[test]
    fn test_read_file() {
        // Create a temporary file for testing
        let file_path = "/tmp/test_scroll.txt";
        let content = "This is test content.";
        fs::write(file_path, content).unwrap();

        let scroll = read_file(file_path, "project_1").unwrap();
        assert_eq!(scroll.scroll_path, file_path);
        assert_eq!(scroll.content, content);
        assert_eq!(scroll.project_id, "project_1");

        // Clean up the temporary file
        fs::remove_file(file_path).unwrap();
    }
}
