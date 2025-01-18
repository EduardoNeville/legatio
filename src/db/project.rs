use sqlx::sqlite::SqlitePool;
use anyhow::Result;
use crate::utils::db_utils::delete_module;
use crate::utils::structs::Project;
use crate::utils::logger::log_error;

/// Inserts a project into the database.
pub async fn store_project(pool: &SqlitePool, project: &Project) -> Result<()> {

    if let Err(error) = sqlx::query("INSERT INTO projects (project_id, project_path) VALUES ($1, $2)")
        .bind(&project.project_id)
        .bind(&project.project_path)
        .execute(pool)
        .await
    {
        log_error(&format!("FAILED :: INSERT project_id: [{}]", 
            project.project_id,
        ));
        return Err(error.into());
    }

    Ok(())
}

/// Fetches all projects from the database.
pub async fn get_projects(pool: &SqlitePool) -> Result<Vec<Project>> {

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
