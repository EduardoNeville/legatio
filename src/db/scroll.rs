use sqlx::sqlite::SqlitePool;
use futures::future;
use anyhow::Result;
use crate::utils::structs::Scroll;
use crate::utils::logger::{log_info, log_error};

/// Inserts a scroll into the database.
pub async fn store_scroll(pool: &SqlitePool, scroll: &Scroll) -> Result<()> {
    log_info("Attempting to store a scroll in the database");

    if let Err(error) = sqlx::query(
        "INSERT INTO scrolls (scroll_id, scroll_path, content, project_id) VALUES ($1, $2, $3, $4)")
        .bind(&scroll.scroll_id)
        .bind(&scroll.scroll_path)
        .bind(&scroll.content)
        .bind(&scroll.project_id)
        .execute(pool)
        .await
    {
        log_error(&format!("Failed to insert scroll: {}", error));
        return Err(error.into());
    }

    Ok(())
}

/// Inserts multiple scrolls into the database in parallel.
pub async fn store_scrolls(pool: &SqlitePool, scrolls: &[Scroll]) -> Result<()> {
    log_info("Attempting to store multiple scrolls");

    let results = future::join_all(scrolls.iter().map(|scroll| {
        let pool = pool.clone();
        async move {
            if let Err(error) = store_scroll(&pool, scroll).await {
                log_error(&format!("Failed to store scroll: {}", error));
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

    log_info("All scrolls stored successfully");
    Ok(())
}

pub async fn get_scrolls(pool: &SqlitePool, project_id: &str)-> Result<Vec<Scroll>> {
    let scrolls_result: Vec<Scroll> = sqlx::query_as::<_, Scroll>(
        "SELECT *
        FROM scrolls 
        WHERE project_id = $1;")
        .bind(project_id)
        .fetch_all(pool)
        .await
        .unwrap();

    Ok(scrolls_result)
}

pub async fn delete_scroll(pool: &SqlitePool, scroll: &Scroll) -> Result<()> {
    log_info(&format!("Deleting scroll: {}", &scroll.scroll_id));

    if let Err(error) =  sqlx::query("DELETE FROM scrolls WHERE scroll_id=$1")
        .bind(&scroll.scroll_id)
        .execute(pool)
        .await
    {
        log_error(&format!("Unable to DELETE scroll: {}", scroll.scroll_id));
        return Err(error.into());
    }
    
    log_info("Successfully deleted scroll");
    Ok(())
}
