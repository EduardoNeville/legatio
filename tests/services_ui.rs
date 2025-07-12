#[cfg(test)]
mod tests {
    use legatio::{
        core::prompt::{get_prompts, store_prompt},
        services::ui::*,
        utils::structs::{Project, Prompt, Scroll},
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
        // Arrange: build a fake project and a scroll under that project path
        let project = Project {
            project_id: "project1".to_string(),
            // note trailing slash so strip_prefix yields exactly "scroll_test"
            project_path: "/fake/project/path/".to_string(),
        };

        let scroll = Scroll {
            scroll_id: "scroll_id_1".to_string(),
            scroll_path: "/fake/project/path/scroll_test".to_string(),
            content: "dummy content".to_string(),
            project_id: "project1".to_string(),
        };

        let scrolls_vec = vec![scroll];

        // Act
        let result = usr_scrolls(scrolls_vec, &project).await;

        // Assert
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 1);
        // we expect a Line containing "scroll_test"
        assert_eq!(lines[0], "scroll_test".into());
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
        let formatted = result.expect("helper_print failed");
        assert!(formatted
            .iter()
            .any(|s| s.contains("  |> Prompt: Root Prompt")));
        assert!(formatted
            .iter()
            .any(|s| s.contains("  |  |> Prompt: Child Prompt")));
        assert!(formatted
            .iter()
            .any(|s| s.contains("  |  |  |> Prompt: Grandchild Prompt")));
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

        // Act
        let formatted_prompts = usr_prompts(&prompts).await;

        // Assert
        let formatted = formatted_prompts.expect("usr_prompts failed");
        // each prompt produces 3 lines: indent, prompt, output → 3 * 3 = 9
        assert_eq!(formatted.len(), 9);
        assert!(formatted
            .iter()
            .any(|s| s.contains("  |> Prompt: Root Prompt")));
        assert!(formatted
            .iter()
            .any(|s| s.contains("  |  |> Prompt: Child Prompt")));
        assert!(formatted
            .iter()
            .any(|s| s.contains("  |  |  |> Prompt: Grandchild Prompt")));
    }

    #[test]
    fn test_usr_prompt_chain() {
        // Arrange
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

        // Act
        let result = usr_prompt_chain(&prompts);

        // Assert: we should have 2 lines per prompt = 6
        assert_eq!(result.len(), 6);
        // Check that each prompt/output appears in reverse
        assert!(result[0].ends_with("Third Prompt"));
        assert!(result[1].ends_with("Third Output"));
        assert!(result[2].ends_with("Second Prompt"));
        assert!(result[3].ends_with("Second Output"));
        assert!(result[4].ends_with("First Prompt"));
        assert!(result[5].ends_with("First Output"));
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

        // Assert
        assert_eq!(prompts.len(), 1);
        let retrieved = &prompts[0];
        assert_eq!(retrieved.prompt_id, prompt.prompt_id);
        assert_eq!(retrieved.content, prompt.content);
        assert_eq!(retrieved.output, prompt.output);
        assert_eq!(retrieved.prev_prompt_id, prompt.prev_prompt_id);
    }
}
