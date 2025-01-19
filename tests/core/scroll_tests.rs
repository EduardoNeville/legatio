use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use src::core::scroll::{store_scroll, get_scrolls, delete_scroll, read_file};
use src::utils::structs::Scroll;
use std::fs;

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

    let scroll = read_file(file_path, "project_1").unwrap();
    assert_eq!(scroll.scroll_path, file_path);
    assert_eq!(scroll.content, content);
    assert_eq!(scroll.project_id, "project_1");

    fs::remove_file(file_path).unwrap();
}
