use crate::utils::{db_utils::delete_module, error::AppError, logger::log_error, structs::Project};
use anyhow::Result;
use ratatui::text::Line;
use sqlx::sqlite::SqlitePool;

/// Inserts a project into the database.
pub async fn store_project(pool: &SqlitePool, project: &Project) -> Result<()> {
    sqlx::query(
        "INSERT INTO projects (project_id, project_path) 
         SELECT $1, $2
         WHERE NOT EXISTS (
             SELECT 1 FROM projects WHERE project_path = $2
         )",
    )
    .bind(&project.project_id)
    .bind(&project.project_path)
    .execute(pool)
    .await
    .map_err(|err| {
        log_error(&format!(
            "FAILED :: INSERT project_id: [{}]",
            project.project_id,
        ));
        AppError::DatabaseError(format!(
            "Failed to store project with ID {}. Reason: {}",
            project.project_id, err
        ))
    })?;

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
    delete_module(pool, "projects", col_name, project_id)
        .await
        .expect("Error in project deletion");

    delete_module(pool, "prompts", col_name, project_id)
        .await
        .expect("Error in prompts deletion");

    delete_module(pool, "scrolls", col_name, project_id)
        .await
        .expect("Error in scoll deletion");

    Ok(())
}

pub fn format_project_title(current_project: &Option<Project>) -> String {
    match current_project {
        Some(project) => format!(
            "[ Current Project: {} ]",
            project.project_path.split('/').next_back().unwrap_or("")
        ),
        None => "[ Projects ]".to_string(),
    }
}

pub fn build_select_project(projects: &[Project]) -> (Vec<Line<'static>>, Vec<String>) {
    let mut proj_items: Vec<Line> = vec![];
    let mut str_items: Vec<String> = vec![];
    for project in projects {
        let proj_name = format!(
            " -[ {} ]-",
            project.project_path.split("/").last().unwrap_or("")
        );
        str_items.push(proj_name.to_owned());
        proj_items.push(Line::from(proj_name));
    }
    (proj_items, str_items)
}
