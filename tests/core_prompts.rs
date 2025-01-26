#[cfg(test)]
mod tests {
    use legatio::{
        core::prompt::{delete_prompt, get_prompts, store_prompt, update_prompt}, prompt_chain, system_prompt, utils::structs::{Prompt, Scroll}
    };
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::SqlitePool;

    async fn create_test_pool() -> SqlitePool {
        // Create a temporary in-memory SQLite database for testing
        SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create database connection pool")
    }

    #[tokio::test]
    async fn test_store_prompt_success() {
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
            prompt_id: "prompt_1".to_string(),
            project_id: "project_1".to_string(),
            prev_prompt_id: "".to_string(),
            content: "content".to_string(),
            output: "output".to_string(),
        };

        let result = store_prompt(&pool, &prompt).await;

        assert!(result.is_ok());

        // Verify that the prompt was stored
        let stored_prompts = get_prompts(&pool, "project_1").await.unwrap();
        assert_eq!(stored_prompts.len(), 1);

        let stored_prompt = &stored_prompts[0];
        assert_eq!(stored_prompt.prompt_id, "prompt_1");
        assert_eq!(stored_prompt.content, "content");
        assert_eq!(stored_prompt.output, "output");
    }

    #[tokio::test]
    async fn test_store_prompt_duplicate() {
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
            prompt_id: "prompt_1".to_string(),
            project_id: "project_1".to_string(),
            prev_prompt_id: "".to_string(),
            content: "content".to_string(),
            output: "output".to_string(),
        };

        store_prompt(&pool, &prompt).await.unwrap();

        // Attempt to store a duplicate prompt
        let duplicate_result = store_prompt(&pool, &prompt).await;

        // The second insertion should succeed silently
        assert!(duplicate_result.is_ok());

        // Ensure that there is still only one prompt stored
        let stored_prompts = get_prompts(&pool, "project_1").await.unwrap();
        assert_eq!(stored_prompts.len(), 1);
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

        let result = update_prompt(
            &pool,
            "content",
            "Updated Content",
            "prompt_id",
            "test_prompt",
        )
        .await;
        assert!(result.is_ok());

        let updated_prompt: String =
            sqlx::query_scalar("SELECT content FROM prompts WHERE prompt_id = 'test_prompt'")
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

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM prompts WHERE prompt_id = 'test_prompt'")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_system_prompt() {
        let scrolls = vec![
            Scroll {
                project_id: "project_1".to_string(),
                scroll_id: "scroll_1".to_string(),
                scroll_path: "/path/to/scroll_1".to_string(),
                content: "Content for Scroll 1".to_string(),
            },
            Scroll {
                project_id: "project_2".to_string(),
                scroll_id: "scroll_2".to_string(),
                scroll_path: "/path/to/scroll_2".to_string(),
                content: "Content for Scroll 2".to_string(),
            },
        ];

        let result = system_prompt(&scrolls).await;

        assert!(result.contains("```"));
        assert!(result.contains("scroll_1"));
        assert!(result.contains("Content for Scroll 1"));
        assert!(result.contains("scroll_2"));
        assert!(result.contains("Content for Scroll 2"));
    }

    #[test]
    fn test_prompt_chain() {
        let prompt1 = Prompt {
            prompt_id: "1".to_string(),
            project_id: "project".to_string(),
            prev_prompt_id: "".to_string(),
            content: "Prompt 1".to_string(),
            output: "Output 1".to_string(),
        };

        let prompt2 = Prompt {
            prompt_id: "2".to_string(),
            project_id: "project".to_string(),
            prev_prompt_id: "1".to_string(),
            content: "Prompt 2".to_string(),
            output: "Output 2".to_string(),
        };

        let prompt3 = Prompt {
            prompt_id: "3".to_string(),
            project_id: "project".to_string(),
            prev_prompt_id: "2".to_string(),
            content: "Prompt 3".to_string(),
            output: "Output 3".to_string(),
        };

        let prompts = vec![prompt1.clone(), prompt2.clone(), prompt3.clone()];

        let chain = prompt_chain(&prompts, &prompt3);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].prompt_id, "3");
        assert_eq!(chain[1].prompt_id, "2");
        assert_eq!(chain[2].prompt_id, "1");
    }

    #[tokio::test]
    async fn test_prompt_integration() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        // Create a temporary table for testing
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

        // Step 1: Create a Prompt
        let prompt = Prompt {
            prompt_id: "prompt_1".to_string(),
            project_id: "project_1".to_string(),
            prev_prompt_id: "prev_prompt".to_string(),
            content: "This is a test prompt".to_string(),
            output: "Test output".to_string(),
        };

        let store_result = store_prompt(&pool, &prompt).await;
        assert!(store_result.is_ok(), "Failed to store prompt");

        // Step 2: Retrieve the Prompt
        let fetched_prompts = get_prompts(&pool, "project_1").await.unwrap();
        assert_eq!(fetched_prompts.len(), 1, "Expected to fetch one prompt");

        let fetched_prompt = &fetched_prompts[0];
        assert_eq!(fetched_prompt.prompt_id, prompt.prompt_id);
        assert_eq!(fetched_prompt.project_id, prompt.project_id);
        assert_eq!(fetched_prompt.prev_prompt_id, prompt.prev_prompt_id);
        assert_eq!(fetched_prompt.content, prompt.content);
        assert_eq!(fetched_prompt.output, prompt.output);

        // Step 3: Update the Prompt
        let update_result = update_prompt(&pool, "content", "Updated prompt content", "prompt_id", "prompt_1").await;
        assert!(update_result.is_ok(), "Failed to update prompt");

        let updated_prompts = get_prompts(&pool, "project_1").await.unwrap();
        assert_eq!(updated_prompts[0].content, "Updated prompt content", "Prompt content was not updated correctly");

        // Step 4: Delete the Prompt
        let delete_result = delete_prompt(&pool, &prompt).await;
        assert!(delete_result.is_ok(), "Failed to delete prompt");

        let remaining_prompts = get_prompts(&pool, "project_1").await.unwrap();
        assert_eq!(remaining_prompts.len(), 0, "Expected no remaining prompts after deletion");
    }
}
