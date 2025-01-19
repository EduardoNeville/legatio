use ratatui::text::Line;
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

pub fn format_project_title(current_project: &Option<Project>) -> String {
    match current_project {
        Some(project) => format!(
            "[ Current Project: {} ]",
            project.project_path.split('/').last().unwrap_or("")
        ),
        None => "[ Projects ]".to_string(),
    }
}

pub fn build_select_project(projects: &[Project])-> (Vec<Line<'static>>, Vec<String>) {
    let mut proj_items: Vec<Line> = vec![];
    let mut str_items: Vec<String> = vec![];
    for project in projects {
        let proj_name = format!(" -[ {} ]-",
            project
            .project_path
            .split("/")
            .last()
            .unwrap_or("")
        );
        println!("proj_name: {}", proj_name.as_str());
        str_items.push(proj_name.to_owned());
        proj_items.push(Line::from(proj_name));
    }
    return (proj_items, str_items)
}

#[cfg(test)]
mod tests {
    use super::*; // Import functions defined in this file
    use sqlx::sqlite::SqlitePoolOptions;
    use crate::utils::{logger::initialize_logger, structs::Project};

    async fn create_test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create database connection pool")
    }

    #[tokio::test]
    async fn test_store_project() {
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE projects (
                project_id TEXT PRIMARY KEY,
                project_path TEXT
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        let project = Project {
            project_id: "project_1".to_string(),
            project_path: "/path/to/project_1".to_string(),
        };

        let result = store_project(&pool, &project).await;
        assert!(result.is_ok());

        // Verify project was stored
        let stored_projects = get_projects(&pool).await.unwrap();
        assert_eq!(stored_projects.len(), 1);

        let stored_project = &stored_projects[0];
        assert_eq!(stored_project.project_id, "project_1");
        assert_eq!(stored_project.project_path, "/path/to/project_1");
    }

    #[tokio::test]
    async fn test_get_projects() {
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE projects (
                project_id TEXT PRIMARY KEY,
                project_path TEXT
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO projects (project_id, project_path) VALUES ('project_1', '/project1'), ('project_2', '/project2')"
        )
        .execute(&pool)
        .await
        .unwrap();

        let projects = get_projects(&pool).await.unwrap();
        assert_eq!(projects.len(), 2);

        assert_eq!(projects[0].project_id, "project_1");
        assert_eq!(projects[0].project_path, "/project1");
        assert_eq!(projects[1].project_id, "project_2");
        assert_eq!(projects[1].project_path, "/project2");
    }

    #[tokio::test]
    async fn test_delete_project() {
        let _ = initialize_logger("test.log");
        let pool = create_test_pool().await;

        sqlx::query(
            "CREATE TABLE projects (
                project_id TEXT PRIMARY KEY,
                project_path TEXT
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE prompts (
                prompt_id TEXT PRIMARY KEY,
                project_id TEXT,
                prev_prompt_id TEXT
            );",)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE scrolls (
                scroll_id TEXT PRIMARY KEY,
                project_id TEXT
            );",)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO projects (project_id, project_path) VALUES ('project_1', '/project1');"
        )
        .execute(&pool)
        .await
        .unwrap();

        let delete_result = delete_project(&pool, "project_1").await;
        assert!(delete_result.is_ok());

        let projects = get_projects(&pool).await.unwrap();
        assert!(projects.is_empty());
    }

    #[test]
    fn test_format_project_title() {
        let project = Some(Project {
            project_id: "project_1".to_string(),
            project_path: "/path/to/project_1".to_string(),
        });

        let result = format_project_title(&project);
        assert_eq!(result, "[ Current Project: project_1 ]");

        let result_no_project = format_project_title(&None);
        assert_eq!(result_no_project, "[ Projects ]");
    }

    #[test]
    fn test_build_select_project() {
        let projects = vec![
            Project {
                project_id: "project_1".to_string(),
                project_path: "/path/to/project_1".to_string(),
            },
            Project {
                project_id: "project_2".to_string(),
                project_path: "/path/to/project_2".to_string(),
            },
        ];

        let (line_items, str_items) = build_select_project(&projects);

        assert_eq!(line_items.len(), 2);
        assert!(str_items.contains(&" -[ project_1 ]-".to_string()));
        assert!(str_items.contains(&" -[ project_2 ]-".to_string()));
    }
}
