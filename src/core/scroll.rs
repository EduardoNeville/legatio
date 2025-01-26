use crate::utils::{
    db_utils::delete_module,
    error::AppError,
    logger::log_error,
    structs::Scroll,
};
use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use std::fs;

/// Inserts a scroll into the database.
pub async fn store_scroll(pool: &SqlitePool, scroll: &Scroll) -> Result<()> {
    sqlx::query(
        "INSERT INTO scrolls (scroll_id, scroll_path, content, project_id) 
         SELECT $1, $2, $3, $4
         WHERE NOT EXISTS (
             SELECT 1 FROM scrolls WHERE scroll_path = $2 AND content = $3
         )"
    )
    .bind(&scroll.scroll_id)
    .bind(&scroll.scroll_path)
    .bind(&scroll.content)
    .bind(&scroll.project_id)
    .execute(pool)
    .await
    .map_err(|err| {
        log_error(&format!(
            "FAILED :: INSERT scroll_id: [{}]",
            scroll.scroll_id,
        ));
        AppError::DatabaseError(format!(
            "Failed to store scroll with ID {}. Reason: {}",
            scroll.scroll_id, err
        ))
    })?;

    Ok(())
}

pub async fn get_scrolls(pool: &SqlitePool, project_id: &str) -> Result<Vec<Scroll>> {
    let scrolls_result: Vec<Scroll> = sqlx::query_as::<_, Scroll>(
        "SELECT *
        FROM scrolls 
        WHERE project_id = $1;",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .unwrap();

    Ok(scrolls_result)
}

pub async fn delete_scroll(pool: &SqlitePool, scroll_id: &str) -> Result<()> {
    delete_module(pool, "scrolls", "scroll_id", scroll_id)
        .await
        .expect("Error in scroll deletion");
    Ok(())
}

pub async fn update_scroll_content(pool: &SqlitePool, scroll: &Scroll) -> Result<Scroll> {
    let new_scroll_result = read_file(&scroll.scroll_path, &scroll.project_id, Some(scroll));

    // Match on the result of reading the file
    match new_scroll_result {
        Ok(new_scroll) => {
            // If the file was read successfully, proceed to update the scroll in the database
            sqlx::query(
                "UPDATE scrolls
                 SET content = $1
                 WHERE scroll_id = $2",
            )
            .bind(&new_scroll.content) // Bind new content
            .bind(&new_scroll.scroll_id) // Use the scroll ID to locate record
            .execute(pool)
            .await
            .map_err(|err| {
                log_error(&format!(
                    "FAILED :: UPDATE scroll_id: {}, error: {}",
                    new_scroll.scroll_id, err
                ));
                AppError::DatabaseError(format!(
                    "Failed to update scroll: {}. Reason: {}",
                    scroll.scroll_id, err
                ))
            })?;

            Ok(new_scroll)
        }
        Err(err) => {
            // Check if the error corresponds to a missing file
            if let Some(err_str) = err.downcast_ref::<std::io::Error>() {
                if err_str.kind() == std::io::ErrorKind::NotFound {
                    // Log that we're deleting the scroll because the file was not found
                    log_error(&format!(
                        "File not found for scroll_id: {}, deleting scroll from database",
                        scroll.scroll_id
                    ));

                    // Delete the scroll from the database
                    delete_scroll(pool, &scroll.scroll_id).await?;
                    return Err(AppError::FileError(format!(
                        "File not found at '{}', scroll deleted from database.",
                        scroll.scroll_path
                    ))
                    .into());
                }
            }

            // If the error is not because of a missing file, propagate it
            Err(AppError::FileError(format!(
                "Failed to read file '{}': {}",
                scroll.scroll_path,
                err
            ))
            .into())
        }
    }
}

pub fn read_file(file_path: &str, project_id: &str, scroll: Option<&Scroll>) -> Result<Scroll> {
    // Attempt to read the file content
    match fs::read_to_string(file_path) {
        Ok(content) => {
            // If a scroll is provided, return it with updated content
            // Otherwise, create a new scroll with the file content
            if let Some(existing_scroll) = scroll {
                Ok(Scroll {
                    scroll_id: existing_scroll.scroll_id.clone(),
                    scroll_path: existing_scroll.scroll_path.clone(),
                    project_id: existing_scroll.project_id.clone(),
                    content, // Updated content
                })
            } else {
                Ok(Scroll::new(file_path, &content, project_id))
            }
        }
        Err(error) => {
            // Handle file not found error
            if error.kind() == std::io::ErrorKind::NotFound {
                Err(AppError::FileError(format!(
                    "File not found at path: '{}'",
                    file_path
                ))
                .into())
            } else {
                // Handle other file-related errors
                Err(AppError::FileError(format!(
                    "Failed to read file at '{}': {}",
                    file_path, error
                ))
                .into())
            }
        }
    }
}
