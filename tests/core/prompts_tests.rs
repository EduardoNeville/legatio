use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use crate::core::prompts::{store_prompt, get_prompts, update_prompt, delete_prompt};
use crate::utils::structs::Prompt;

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
