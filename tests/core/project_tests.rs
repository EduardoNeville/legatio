use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use src::core::project::{store_project, get_projects, delete_project};
use src::utils::structs::Project;

#[tokio::test]
async fn test_project_full_integration() {
    // Step 1: Setup in-memory database
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE projects (
            project_id TEXT PRIMARY KEY,
            project_path TEXT
        );",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Step 2: Store a Project
    let project = Project {
        project_id: "project_1".to_string(),
        project_path: "/path/to/project_1".to_string(),
    };

    let store_result = store_project(&pool, &project).await;
    assert!(store_result.is_ok(), "Failed to store project");

    // Step 3: Retrieve Stored Project
    let projects = get_projects(&pool).await.unwrap();
    assert_eq!(projects.len(), 1);

    let retrieved_project = &projects[0];
    assert_eq!(retrieved_project.project_id, "project_1");
    assert_eq!(retrieved_project.project_path, "/path/to/project_1");

    // Step 4: Add Another Project
    let project_2 = Project {
        project_id: "project_2".to_string(),
        project_path: "/path/to/project_2".to_string(),
    };

    let store_result_2 = store_project(&pool, &project_2).await;
    assert!(store_result_2.is_ok(), "Failed to store second project");

    let projects = get_projects(&pool).await.unwrap();
    assert_eq!(projects.len(), 2);

    // Step 5: Delete a Project
    let delete_result = delete_project(&pool, "project_1").await;
    assert!(delete_result.is_ok(), "Failed to delete project");

    let projects = get_projects(&pool).await.unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].project_id, "project_2");
    assert_eq!(projects[0].project_path, "/path/to/project_2");
}
