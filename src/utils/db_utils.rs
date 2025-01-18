use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};
use anyhow::Result;
use crate::utils::logger::{log_info, log_error};

pub async fn get_db_pool(db_url: &str) -> Result<SqlitePool> {
    if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
        match Sqlite::create_database(db_url).await {
            Ok(_) => log_info("Create db success"),
            Err(error) => panic!("error: {}", error),
        }

        let db = SqlitePool::connect(db_url).await.unwrap();
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let migrations = std::path::Path::new(&crate_dir).join("./migrations");
        let migration_results = sqlx::migrate::Migrator::new(migrations)
            .await
            .unwrap()
            .run(&db)
            .await;
        match migration_results {
            Ok(_) => log_info("Migration success"),
            Err(error) => {
                panic!("error: {}", error);
            }
        }
    } 
    
    let db = SqlitePool::connect(db_url).await.unwrap();
    Ok(db)
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
        log_error(&format!("FAILED :: DELETE {}: [{}]", 
            column_name,
            column_value
        ));
        return Err(error.into());
    }
    Ok(())
}
