use sqlx::sqlite::SqlitePool;
use anyhow::Result;
use crate::utils::structs::Project;
use crate::utils::logger::{log_info, log_error};

/// Inserts a project into the database.
pub async fn store_project(pool: &SqlitePool, project: &Project) -> Result<()> {
    log_info("Attempting to store a new project into the database");

    if let Err(error) = sqlx::query("INSERT INTO projects (project_id, project_path) VALUES ($1, $2)")
        .bind(&project.project_id)
        .bind(&project.project_path)
        .execute(pool)
        .await
    {
        log_error(&format!("Failed to insert project: {}", error));
        return Err(error.into());
    }

    log_info(&format!("Insert successfull of project: {}", project.project_id));
    Ok(())
}

/// Fetches all projects from the database.
pub async fn get_projects(pool: &SqlitePool) -> Result<Vec<Project>> {
    log_info("Fetching all projects from the database");

    let result = sqlx::query_as::<_, Project>("SELECT * FROM projects;")
        .fetch_all(pool)
        .await;

    match result {
        Ok(projects) => Ok(projects),
        Err(error) => {
            log_error(&format!("Failed to fetch projects: {}", error));
            Err(error.into())
        }
    }
}

pub async fn delete_module(
    pool: &SqlitePool,
    table: &str,
    column_name: &str,
    column_value: &str,
) -> Result<()> {
    // Construct the query dynamically
    let query = format!("DELETE FROM {} WHERE {} = ?", table, column_name);

    // Execute the query with the given value as a parameter
    if let Err(error) = sqlx::query(&query)
        .bind(column_value)
        .execute(pool)
        .await
    {
        log_error(&format!(
            "FAILED :: DELETE from {} where {} = {}",
            table, column_name, column_value
        ));
        return Err(error.into());
    }

    log_info(&format!(
        "SUCCESSFUL :: DELETE from {} where {} = {}",
        table, column_name, column_value
    ));
    Ok(())
}

pub async fn delete_project(pool: &SqlitePool, project_id: &str) -> Result<()> {
    let col_name = "project_id";
    delete_module(pool, &"projects", &col_name, project_id)
        .await
        .expect("Error in project deletion");

    
    delete_module(pool, &"prompts", &col_name, project_id)
        .await
        .expect("Error in prompts deletion");

    delete_module(pool, &"scrolls", &col_name, project_id)
        .await
        .expect("Error in scoll deletion");

    Ok(())
}
