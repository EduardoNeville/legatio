#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;
    use legatio::{
        core::scroll::{delete_scroll, get_scrolls, read_file, store_scroll}, update_scroll_content, utils::structs::Scroll, AppError
    };
    use std::fs;

    // Utility function to create an in-memory SQLite pool for testing
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

    #[tokio::test]
    async fn test_update_scroll_content() {
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

        // Create a test file for the scroll
        let file_path = "/tmp/test_scroll_update.txt";
        let initial_content = "Initial test content.";
        fs::write(file_path, initial_content).unwrap();

        // Insert the scroll into the database
        let scroll = Scroll {
            scroll_id: "scroll_1".to_string(),
            scroll_path: file_path.to_string(),
            content: initial_content.to_string(),
            project_id: "project_1".to_string(),
        };

        store_scroll(&pool, &scroll).await.unwrap();

        // Update the file's content
        let new_content = "Updated test content.";
        fs::write(file_path, new_content).unwrap();

        // Update the scroll in the database
        let updated_scroll = update_scroll_content(&pool, &scroll).await.unwrap();

        // Verify the updated scroll
        assert_eq!(updated_scroll.content, new_content);

        // Verify that the database reflects the updates
        let stored_scrolls = get_scrolls(&pool, "project_1").await.unwrap();
        assert_eq!(stored_scrolls.len(), 1);
        assert_eq!(stored_scrolls[0].content, new_content);

        // Clean up test file
        fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_read_file() {
        // Case 1: File exists and can be read successfully
        let file_path = "/tmp/test_read_file_success.txt";
        let content = "This is test content for read_file.";
        fs::write(file_path, content).unwrap();

        let scroll = read_file(file_path, "project_1", None).unwrap();
        assert_eq!(scroll.scroll_path, file_path);
        assert_eq!(scroll.content, content);
        assert_eq!(scroll.project_id, "project_1");
        
        // Clean up the temporary file
        fs::remove_file(file_path).unwrap();

        // Case 2: File does not exist
        let invalid_file_path = "/tmp/invalid_read_file.txt";
        let error_result = read_file(invalid_file_path, "project_1", None);

        assert!(error_result.is_err());
        if let Some(AppError::FileError(err_msg)) = error_result.err().unwrap().downcast_ref::<AppError>() {
            assert!(err_msg.contains("File not found at path"));
        }
    }

    #[tokio::test]
    async fn test_scroll_full_integration() {
        // Step 1: Setup an in-memory database
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

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

        // Step 2: Add a scroll
        let scroll = Scroll {
            scroll_id: "scroll_1".to_string(),
            scroll_path: "/path/to/scroll_1".to_string(),
            content: "Test Scroll Content".to_string(),
            project_id: "project_1".to_string(),
        };

        let store_result = store_scroll(&pool, &scroll).await;
        assert!(store_result.is_ok(), "Failed to store scroll");

        // Step 3: Retrieve the scroll
        let scrolls = get_scrolls(&pool, "project_1").await.unwrap();
        assert_eq!(scrolls.len(), 1);

        let stored_scroll = &scrolls[0];
        assert_eq!(stored_scroll.scroll_id, "scroll_1");
        assert_eq!(stored_scroll.scroll_path, "/path/to/scroll_1");
        assert_eq!(stored_scroll.content, "Test Scroll Content");
        assert_eq!(stored_scroll.project_id, "project_1");

        // Step 4: Add another scroll
        let scroll_2 = Scroll {
            scroll_id: "scroll_2".to_string(),
            scroll_path: "/path/to/scroll_2".to_string(),
            content: "Second Scroll Content".to_string(),
            project_id: "project_1".to_string(),
        };

        store_scroll(&pool, &scroll_2).await.unwrap();

        let scrolls = get_scrolls(&pool, "project_1").await.unwrap();
        assert_eq!(scrolls.len(), 2);

        // Step 5: Delete a scroll
        let delete_result = delete_scroll(&pool, "scroll_1").await;
        assert!(delete_result.is_ok(), "Failed to delete scroll");

        let scrolls = get_scrolls(&pool, "project_1").await.unwrap();
        assert_eq!(scrolls.len(), 1);
        assert_eq!(scrolls[0].scroll_id, "scroll_2");
    }

    #[test]
    fn test_read_file_integration() {
        let file_path = "/tmp/integration_test_scroll.txt";
        let content = "Integration test scroll content";

        fs::write(file_path, content).unwrap();

        let scroll = read_file(file_path, "project_1", None).unwrap();
        assert_eq!(scroll.scroll_path, file_path);
        assert_eq!(scroll.content, content);
        assert_eq!(scroll.project_id, "project_1");

        fs::remove_file(file_path).unwrap();
    }
}
