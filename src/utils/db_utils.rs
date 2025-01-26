use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};
use anyhow::Result;
use crate::utils::logger::{log_info, log_error};

use super::error::AppError;

pub async fn get_db_pool(db_url: &str) -> Result<SqlitePool, AppError> {
    // Check if database exists, if not, create it
    if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
        match Sqlite::create_database(db_url).await {
            Ok(_) => log_info("Database created successfully."),
            Err(error) => {
                let error_msg = format!("Failed to create database: {}", error);
                log_error(&error_msg);
                return Err(AppError::DatabaseError(error_msg));
            }
        }
        
        // Connect to the database
        let pool = match SqlitePool::connect(db_url).await {
            Ok(pool) => pool,
            Err(error) => {
                let error_msg = format!("Failed to connect to the database: {}", error);
                log_error(&error_msg);
                return Err(AppError::DatabaseError(error_msg));
            }
        };

        // Create the required tables
        let create_projects_table = r#"
            CREATE TABLE IF NOT EXISTS projects (
                project_id TEXT PRIMARY KEY,
                project_path TEXT
            );
        "#;

        let create_scrolls_table = r#"
            CREATE TABLE IF NOT EXISTS scrolls (
                scroll_id TEXT PRIMARY KEY,
                scroll_path TEXT,
                content TEXT,
                project_id TEXT
            );
        "#;

        let create_prompts_table = r#"
            CREATE TABLE IF NOT EXISTS prompts (
                prompt_id TEXT,
                project_id TEXT,
                content TEXT,
                output TEXT,
                prev_prompt_id TEXT
            );
        "#;

        if let Err(error) = sqlx::query(create_projects_table).execute(&pool).await {
            let error_msg = format!("Failed to create projects table: {}", error);
            log_error(&error_msg);
            return Err(AppError::DatabaseError(error_msg));
        } else {
            log_info("Projects table created (if not already present).");
        }

        if let Err(error) = sqlx::query(create_scrolls_table).execute(&pool).await {
            let error_msg = format!("Failed to create scrolls table: {}", error);
            log_error(&error_msg);
            return Err(AppError::DatabaseError(error_msg));
        } else {
            log_info("Scrolls table created (if not already present).");
        }

        if let Err(error) = sqlx::query(create_prompts_table).execute(&pool).await {
            let error_msg = format!("Failed to create prompts table: {}", error);
            log_error(&error_msg);
            return Err(AppError::DatabaseError(error_msg));
        } else {
            log_info("Prompts table created (if not already present).");
        }
    }
    
    let pool = match SqlitePool::connect(db_url).await {
        Ok(pool) => pool,
        Err(error) => {
            let error_msg = format!("Failed to connect to the database: {}", error);
            log_error(&error_msg);
            return Err(AppError::DatabaseError(error_msg));
        }
    };

    Ok(pool)
}

pub async fn delete_module(
    pool: &SqlitePool,
    table: &str,
    column_name: &str,
    column_value: &str,
) -> Result<(), AppError> {
    // Construct the query dynamically
    let query = format!("DELETE FROM {} WHERE {} = ?", table, column_name);

    // Execute the query with the given value as a parameter
    match sqlx::query(&query)
        .bind(column_value)
        .execute(pool)
        .await
    {
        Ok(_) => {
            log_info(&format!(
                "Successfully deleted row from {} where {} = [{}]",
                table, column_name, column_value
            ));
            Ok(())
        }
        Err(error) => {
            let error_msg = format!(
                "Failed to delete from {}: {} = [{}]: {}",
                table, column_name, column_value, error
            );
            log_error(&error_msg);
            Err(AppError::DatabaseError(error_msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    // A test database URL (using an in-memory SQLite database for isolated testing)
    const TEST_DATABASE_URL: &str = "sqlite::memory:";

    // Helper function to set up the database
    async fn setup_test_db() -> SqlitePool {
        // Call the function to create the database and tables
        

        get_db_pool(TEST_DATABASE_URL)
            .await
            .expect("Failed to set up test database")
    }

    #[tokio::test]
    async fn test_get_db_pool_creates_tables() {
        let pool = setup_test_db().await;

        // Here, we'll check if the tables actually exist in the in-memory database
        let result_projects: Result<(String,), _> = sqlx::query_as("SELECT name FROM sqlite_master WHERE type='TABLE' AND name='projects'")
            .fetch_one(&pool)
            .await;

        let result_scrolls: Result<(String,), _> = sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' AND name='scrolls'")
            .fetch_one(&pool)
            .await;

        let result_prompts: Result<(String,), _> = sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' AND name='prompts'")
            .fetch_one(&pool)
            .await;

        assert!(result_projects.is_ok(), "Projects table was not created as expected: \n{:?}", result_projects);
        assert!(result_scrolls.is_ok(), "Scrolls table was not created as expected");
        assert!(result_prompts.is_ok(), "Prompts table was not created as expected");
    }

    //#[tokio::test]
    //async fn test_delete_module_success() {
    //    let pool = setup_test_db().await;

    //    // Insert a dummy record into the projects table
    //    let project_id = "test_project_id";
    //    sqlx::query("INSERT INTO projects (project_id, project_path) VALUES (?, ?)")
    //        .bind(project_id)
    //        .bind("/test/path")
    //        .execute(&pool)
    //        .await
    //        .expect("Failed to insert dummy project");

    //    // Verify the record exists
    //    let project_exists = sqlx::query_as::<_>("SELECT COUNT(*) FROM projects WHERE project_id = ?")
    //        .bind(project_id)
    //        .fetch_one(&pool)
    //        .await
    //        .unwrap()

    //    assert_eq!(project_exists, 1, "Dummy project record was not added");

    //    // Delete the record using the function
    //    delete_module(&pool, "projects", "project_id", project_id)
    //        .await
    //        .expect("Failed to delete project");

    //    // Verify the record no longer exists
    //    let project_exists = sqlx::query("SELECT COUNT(*) FROM projects WHERE project_id = ?")
    //        .bind(project_id)
    //        .fetch_one(&pool)
    //        .await
    //        .expect("Failed to verify project non-existence")
    //        .get::<i64, _>("COUNT(*)");

    //    assert_eq!(project_exists, 0, "Project record was not successfully deleted");
    //}

    //#[tokio::test]
    //async fn test_delete_module_non_existent_record() {
    //    let pool = setup_test_db().await;

    //    let result = delete_module(&pool, "projects", "project_id", "non_existent_id").await;

    //    // Expect no error as the function should handle the "no record found" case gracefully
    //    assert!(result.is_ok(), "delete_module should not fail when deleting a non-existent record");
    //}

    #[tokio::test]
    async fn test_delete_module_errors_on_invalid_table() {
        let pool = setup_test_db().await;

        // Attempt to delete from an invalid table name
        let result = delete_module(&pool, "non_existent_table", "column_name", "value").await;

        // Expect an error since the table doesn't exist
        assert!(
            result.is_err(),
            "delete_module should fail when the table name is invalid"
        );

        // Verify the specific error type
        if let Err(AppError::DatabaseError(message)) = result {
            assert!(message.contains("no such table"), "Error message does not mention 'no such table'");
        } else {
            panic!("Expected AppError::DatabaseError, but received a different error.");
        }
    }

    #[tokio::test]
    async fn test_get_db_pool_handles_reconnection() {
        // Test reconnection to an already-existing database
        {
            let first_pool = get_db_pool(TEST_DATABASE_URL).await.expect("Failed to create first database connection");
            assert!(!first_pool.is_closed(), "First pool is unexpectedly closed");
        }

        // Simulate reconnecting
        {
            let second_pool = get_db_pool(TEST_DATABASE_URL).await.expect("Failed to reconnect to the database");
            assert!(!second_pool.is_closed(), "Second pool is unexpectedly closed");
        }
    }

    #[tokio::test]
    async fn test_db_error_handling_on_failed_connection() {
        // Provide an invalid database URL to simulate a failure
        let invalid_db_url = "sqlite:/invalid_path";

        let result = get_db_pool(invalid_db_url).await;

        // Expect an error
        assert!(
            result.is_err(),
            "get_db_pool should fail when provided with an invalid database URL"
        );

        // Verify the specific error type
        if let Err(AppError::DatabaseError(message)) = result {
            assert!(message.contains("Failed to connect"), "Error message does not mention 'Failed to connect'");
        } else {
            panic!("Expected AppError::DatabaseError, but received a different error.");
        }
    }
}
