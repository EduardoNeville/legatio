#[cfg(test)]
mod tests {
    use legatio::{
        core::prompt::{get_prompts, store_prompt},
        services::ui::*,
        utils::structs::{Project, Prompt},
    };
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

    /// Helper function to create a mock SQLite in-memory database and connection pool.
    async fn create_test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create SQLite in-memory database")
    }

    #[tokio::test]
    async fn test_usr_scrolls() {
        // Arrange: Create mock database and scrolls table
        let pool = create_test_pool().await;
        sqlx::query("CREATE TABLE scrolls (scroll_id TEXT, scroll_path TEXT, content TEXT, project_id TEXT);")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO scrolls (scroll_id, scroll_path, content, project_id) VALUES (?, ?, ?, ?);")
            .bind("scroll_id_1")
            .bind("/path/to/scroll_test")
            .bind("This is a mock scroll's content")
            .bind("project1")
            .execute(&pool)
            .await
            .expect("Failed to insert scroll into database");

        let project = Project {
            project_id: "project1".to_string(),
            project_path: "/fake/project/path".to_string(),
        };

        // Act: Fetch scrolls for the project
        let result = usr_scrolls(&pool, &project).await;

        // Assert: Verify scroll retrieval works correctly
        assert!(result.is_ok());
        let scrolls = result.unwrap();
        assert_eq!(scrolls.len(), 1);
        assert_eq!(scrolls[0], "scroll_test");
    }

    // Testing helper_print recursive formatting with multiple nested prompts
    #[test]
    fn test_helper_print() {
        // Arrange: Create a chain of nested prompts
        let prompt1 = Prompt::new("project1", "Root Prompt", "Root Output", "root");
        let prompt2 = Prompt::new(
            "project1",
            "Child Prompt",
            "Child Output",
            &prompt1.prompt_id,
        );
        let prompt3 = Prompt::new(
            "project1",
            "Grandchild Prompt",
            "Grandchild Output",
            &prompt2.prompt_id,
        );
        let prompts = vec![prompt1.clone(), prompt2.clone(), prompt3.clone()];

        // Act: Format the prompts recursively
        let result = helper_print(&prompts, &prompt1, "  |");

        // Assert: Verify the formatting is correct at each level of recursion
        assert!(result.is_ok());
        let formatted_prompts = result.unwrap();
        assert!(formatted_prompts.contains(&"  |> Prompt: Root Prompt".to_string()));
        assert!(formatted_prompts.contains(&"  |  |> Prompt: Child Prompt".to_string()));
        assert!(formatted_prompts.contains(&"  |  |  |> Prompt: Grandchild Prompt".to_string()));
    }

    // Testing recursive prompt formatting via usr_prompts() function
    #[tokio::test]
    async fn test_usr_prompts() {
        // Arrange: Create mock prompts for a project
        let prompt1 = Prompt::new("project1", "Root Prompt", "Root Output", "project1");
        let prompt2 = Prompt::new(
            "project1",
            "Child Prompt",
            "Child Output",
            &prompt1.prompt_id,
        );
        let prompt3 = Prompt::new(
            "project1",
            "Grandchild Prompt",
            "Grandchild Output",
            &prompt2.prompt_id,
        );
        let prompts = vec![prompt1.clone(), prompt2.clone(), prompt3.clone()];

        // Act: Format all prompts recursively
        let formatted_prompts = usr_prompts(&prompts).await;

        // Assert: Verify all prompts are correctly formatted and included
        assert!(formatted_prompts.is_ok());
        let formatted = formatted_prompts.unwrap();
        assert_eq!(formatted.len(), 9); // "Indentation + Prompt + Output" for each prompt
        assert!(formatted.contains(&"  |> Prompt: Root Prompt".to_string()));
        assert!(formatted.contains(&"  |  |> Prompt: Child Prompt".to_string()));
        assert!(formatted.contains(&"  |  |  |> Prompt: Grandchild Prompt".to_string()));
    }

    #[test]
    fn test_usr_prompt_chain() {
        // Arrange: Create a chain of prompts
        let prompt1 = Prompt::new("project1", "First Prompt", "First Output", "root");
        let prompt2 = Prompt::new(
            "project1",
            "Second Prompt",
            "Second Output",
            &prompt1.prompt_id,
        );
        let prompt3 = Prompt::new(
            "project1",
            "Third Prompt",
            "Third Output",
            &prompt2.prompt_id,
        );
        let prompts = vec![prompt1.clone(), prompt2.clone(), prompt3.clone()];

        // Act: Fetch the reverse-ordered prompt chain
        let result = usr_prompt_chain(&prompts);

        // Assert: Verify the prompts and outputs are in reverse order
        let prompt = " |- Prompt:";
        let output = " |  Output:";
        assert_eq!(result.len(), 6); // Includes Prompt + Output for each
        assert_eq!(result[0], format!("{prompt} Third Prompt"));
        assert_eq!(result[1], format!("{output} Third Output"));
        assert_eq!(result[2], format!("{prompt} Second Prompt"));
        assert_eq!(result[3], format!("{output} Second Output"));
        assert_eq!(result[4], format!("{prompt} First Prompt"));
        assert_eq!(result[5], format!("{output} First Output"));
    }

    #[tokio::test]
    async fn test_store_and_retrieve_prompt_integration() {
        // Arrange: Set up an in-memory database
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

        let prompt = Prompt::new("project1", "Test Content", "Test Output", "root");

        // Act: Store the prompt and retrieve it
        store_prompt(&pool, &prompt).await.unwrap();
        let prompts = get_prompts(&pool, "project1").await.unwrap();

        // Assert: Verify the prompt was stored and retrieved correctly
        assert_eq!(prompts.len(), 1);
        let retrieved_prompt = &prompts[0];
        assert_eq!(retrieved_prompt.prompt_id, prompt.prompt_id);
        assert_eq!(retrieved_prompt.content, "Test Content");
        assert_eq!(retrieved_prompt.output, "Test Output");
        assert_eq!(retrieved_prompt.prev_prompt_id, "root");
    }
}
