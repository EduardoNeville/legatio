use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};
use anyhow::Result;

pub async fn get_db_pool(db_url: &str) -> Result<SqlitePool> {
    if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
        println!("Creating database {}", db_url);
        match Sqlite::create_database(db_url).await {
            Ok(_) => println!("Create db success"),
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
            Ok(_) => println!("Migration success"),
            Err(error) => {
                panic!("error: {}", error);
            }
        }
        println!("migration: {:?}", migration_results);
    } else {
        println!("Database already exists");
    }
    let db = SqlitePool::connect(db_url).await.unwrap();
    Ok(db)
}

