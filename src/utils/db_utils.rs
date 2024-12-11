use crate::utils::structs::{Project, File, Prompt, Scrolls};
use sqlx::{sqlite::SqlitePool, Error};
use anyhow::{Ok, Result};
use std::collections::HashSet;

pub async fn get_db_pool() -> Result<SqlitePool> {
    let pool = SqlitePool::connect("sqlite://legatio.db").await?;
    Ok(pool)
}

pub async fn store_project(pool: &SqlitePool, project: &Project)->Result<()>{
    sqlx::query(
    "INSERT INTO projects (id, project_path, files) 
    VALUES ($1, $2, $3)")
    .bind(project.project_id.clone())
    .bind(project.project_path.clone())
    .bind(project.files.clone())
    .execute(pool).await?;
    Ok(())
}

pub async fn store_file(pool: &SqlitePool, file: &File) -> Result<()> {
    sqlx::query(
    "INSERT INTO files (id, file_path, content)
    VALUES ($1, $2, $3)")
    .bind(file.file_id.clone())
    .bind(file.file_path.clone())
    .bind(file.content.clone())
    .execute(pool).await?;
    Ok(())
}

//pub async fn store_files(pool: &SqlitePool, project_id: &String, files: &Vec<File>)-> Result<()>{
//    let existing_project: Option<Project> = sqlx::query!(
//        "SELECT project_id, project_path, files FROM projects WHERE project_id = ?",
//        project_id
//    )
//    .fetch_optional(pool)
//    .await?
//    .map(|row| Project {
//        project_id: row.project_id,
//        project_path: row.project_path,
//        files: row.files.unwrap_or_else(|| String::new()), // Handle potential NULL in the database
//    });
//
//    let mut existing_file_ids = HashSet::new();
//    if let Some(project) = existing_project {
//        if !project.files.is_empty() {
//            existing_file_ids = project.files.split(':').map(|s| s.to_string()).collect();
//        }
//    }
//
//
//    //Filter the files, only include files whose id is not in the existing files
//    let files_to_insert: Vec<&File> = files
//        .iter()
//        .filter(|file| !existing_file_ids.contains(&file.file_id))
//        .collect();
//
//
//    let mut query = sqlx::query("INSERT INTO files (id, file_path, content) VALUES (?, ?, ?)"); 
//
//    Ok(())
//}

pub async fn store_prompt(pool: &SqlitePool, prompt: &Prompt) -> Result<()> {
    sqlx::query(
        "INSERT INTO prompts (id, prompt, output)
        VALUES ($1, $2, $3)"
    )
    .bind(prompt.prompt_id.clone())
    .bind(prompt.content.clone())
    .bind(prompt.output.clone())
    .execute(pool).await?;
    Ok(())
}

pub async fn store_scrolls(pool: &SqlitePool, scrolls: &Scrolls) -> Result<()> {
    sqlx::query(
        "INSERT INTO scrolls (id, project_id, prompts)
        VALUES ($1, $2, $3)"
    )
    .bind(scrolls.scroll_id.clone())
    .bind(scrolls.project_id.clone())
    .bind(scrolls.prompts.clone())
    .execute(pool).await?;
    Ok(())
}

pub async fn append_id(pool: &SqlitePool, table_id: &u32, column_name: &String, id: &u32)-> Result<()> {
    // TODO
    let ids = sqlx::query("
        SELECT $1 FROM $2 ;
    ").bind(&column_name).bind(table_id)
    .execute(pool).await?;

    println!("ids: {:?}", ids);
    Ok(())
}

