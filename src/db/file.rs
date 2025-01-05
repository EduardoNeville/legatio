use sqlx::sqlite::SqlitePool;
use futures::future;
use anyhow::Result;
use crate::utils::structs::File;
use crate::utils::logger::{log_info, log_error};

/// Inserts a file into the database.
pub async fn store_file(pool: &SqlitePool, file: &File) -> Result<()> {
    log_info("Attempting to store a file in the database");

    if let Err(error) = sqlx::query(
        "INSERT INTO files (file_id, file_path, content, project_id) VALUES ($1, $2, $3, $4)")
        .bind(&file.file_id)
        .bind(&file.file_path)
        .bind(&file.content)
        .bind(&file.project_id)
        .execute(pool)
        .await
    {
        log_error(&format!("Failed to insert file: {}", error));
        return Err(error.into());
    }

    Ok(())
}

/// Inserts multiple files into the database in parallel.
pub async fn store_files(pool: &SqlitePool, files: &[File]) -> Result<()> {
    log_info("Attempting to store multiple files");

    let results = future::join_all(files.iter().map(|file| {
        let pool = pool.clone();
        async move {
            if let Err(error) = store_file(&pool, file).await {
                log_error(&format!("Failed to store file: {}", error));
                Err(error)
            } else {
                Ok(())
            }
        }
    }))
    .await;

    for result in results {
        if let Err(error) = result {
            return Err(error);
        }
    }

    log_info("All files stored successfully");
    Ok(())
}

pub async fn get_files(pool: &SqlitePool, project_id: &str)-> Result<Vec<File>> {
    let files_result: Vec<File> = sqlx::query_as::<_, File>(
        "SELECT *
        FROM files 
        WHERE project_id = $1;")
        .bind(project_id)
        .fetch_all(pool)
        .await
        .unwrap();

    Ok(files_result)
}
