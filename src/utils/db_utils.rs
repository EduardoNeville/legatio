use crate::{
    services::config::get_config_dir,
    utils::logger::{log_error, log_info},
};
use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};

use super::error::AppError;

/// Get the path of the database file inside the Legatio directory.
fn get_db_path() -> Result<String> {
    let config_dir = get_config_dir()?;
    let db_url = format!("sqlite://{}/legatio.db", config_dir.to_string_lossy());
    Ok(db_url)
}

pub async fn get_db_pool() -> Result<SqlitePool, AppError> {
    let db_url = &get_db_path().unwrap();
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
        }

        if let Err(error) = sqlx::query(create_scrolls_table).execute(&pool).await {
            let error_msg = format!("Failed to create scrolls table: {}", error);
            log_error(&error_msg);
            return Err(AppError::DatabaseError(error_msg));
        }

        if let Err(error) = sqlx::query(create_prompts_table).execute(&pool).await {
            let error_msg = format!("Failed to create prompts table: {}", error);
            log_error(&error_msg);
            return Err(AppError::DatabaseError(error_msg));
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
    match sqlx::query(&query).bind(column_value).execute(pool).await {
        Ok(_) => Ok(()),
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
