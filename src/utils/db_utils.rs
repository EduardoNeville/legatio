use sqlx::sqlite::SqlitePool;
use anyhow::Result;

pub async fn get_db_pool() -> Result<SqlitePool> {
    let pool = SqlitePool::connect("sqlite:mydb.db").await?;
    Ok(pool)
}

pub async fn store_prompt(pool: &SqlitePool, user_input: &str) -> Result<()> {
    let query = format!("INSERT INTO prompts (user_input) VALUES ({})", user_input);
    sqlx::query(&query)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn store_files(pool: &SqlitePool, files: &[(String, String)]) -> Result<()> {
    for (path, content) in files {
        let query = format!("INSERT INTO files (path, content) VALUES ({}, {})",
            path,
            content
        );
        sqlx::query(&query)
        .execute(pool)
        .await?;
    }
    Ok(())
}
