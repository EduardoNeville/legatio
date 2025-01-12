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

    log_info("Project insertion successful");
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

pub async fn delete_project(pool: &SqlitePool, project_id: &str) -> Result<()> {
    log_info(&format!("Deleting project: {}", &project_id));

    if let Err(error) =  sqlx::query("DELETE FROM projects WHERE project_id = $1")
        .bind(&project_id)
        .execute(pool)
        .await
    {
        log_error(&format!("Unable to DELETE project: {}", project_id));
        return Err(error.into());
    }
    
    log_info("Successfully deleted project");
    Ok(())
}
