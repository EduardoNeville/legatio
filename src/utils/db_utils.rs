use futures::future;
use crate::utils::structs::{Project, File, Scroll, Prompt};
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

pub async fn store_project(pool: &SqlitePool, project: &Project) -> Result<()> {
    let results = sqlx::query(
        "INSERT INTO projects (project_id, project_path) 
        VALUES ($1, $2)",)
        .bind(&project.project_id)
        .bind(&project.project_path)
        .execute(pool)
        .await
        .unwrap();

    println!("Inserted into projects: {:?}", results);
    Ok(())
}

pub async fn get_projects(pool: &SqlitePool) -> Result<Vec<Project>> {
    let result: Vec<Project> = sqlx::query_as::<_, Project>(
        "SELECT * 
        FROM projects;")
        .fetch_all(pool)
        .await
        .unwrap();
    Ok(result)
}

pub async fn store_file(pool: &SqlitePool, file: &File) -> Result<()> {
    sqlx::query(
        "INSERT INTO files (file_id, file_path, content, project_id) 
        VALUES ($1, $2, $3, $4)")
        .bind(&file.file_id)
        .bind(&file.file_path)
        .bind(&file.content)
        .bind(&file.project_id)
        .execute(pool)
        .await
        .unwrap();

    Ok(())
}


pub async fn store_files(pool: &SqlitePool, files: &[File]) -> Result<()> {
    future::join_all(
        files.into_iter().map(|file| {
            //let pool = pool.clone(); // Clone pool for safety
            async move { store_file(&pool, &file).await }
        })
    )
    .await
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?; // Aggregate results & propagate errors

    Ok(())
}

pub async fn get_files(pool: &SqlitePool, project_id: &String)-> Result<Vec<File>> {
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

pub async fn store_prompt(pool: &SqlitePool, prompt: &Prompt) -> Result<()> {

    sqlx::query(
        "INSERT INTO prompts (prompt_id, scroll_id, content, output, next_prompt_id) 
         VALUES ($1, $2, $3, $4, $5)")
        .bind(&prompt.prompt_id)
        .bind(&prompt.scroll_id)
        .bind(&prompt.content)
        .bind(&prompt.output)
        .bind("")
        .execute(pool)
        .await?;

    Ok(())

}

pub async fn get_scrolls(pool: &SqlitePool, project_id: &String) -> Result<Vec<Scroll>> {
    let result = sqlx::query_as::<_, Scroll>(
        "SELECT * 
        FROM scrolls
        WHERE project_id = $1;")
        .bind(project_id)
        .fetch_all(pool)
        .await
        .unwrap();

    Ok(result)
}


pub async fn get_scroll(pool: &SqlitePool, scroll_id: &String) -> Result<Vec<Scroll>> {
    let result = sqlx::query_as::<_, Scroll>(
        "SELECT * 
        FROM scrolls
        WHERE scroll_id = $1;")
        .bind(scroll_id)
        .fetch_all(pool)
        .await
        .unwrap();

    Ok(result)
}

pub async fn store_scroll(pool: &SqlitePool, scroll: &Scroll) -> Result<()> {
    sqlx::query(
        "INSERT INTO scrolls (scroll_id, project_id, init_prompt_id) 
        VALUES ($1, $2, $3)")
        .bind(&scroll.scroll_id)
        .bind(&scroll.project_id)
        .bind(&scroll.init_prompt_id)
        .execute(pool)
        .await?;

    Ok(())
}

// Updates scrolls init_prompt_id if "" aka no prompts
// Updates the last prompt in the list to the new prompt id
pub async fn update_scroll(pool: &SqlitePool, scroll: &Scroll, prompt: &Prompt) -> Result<()> {
    if scroll.init_prompt_id == "" {
        sqlx::query(
            "UPDATE scrolls 
            SET init_prompt_id = $1 
            WHERE scroll_id = $2")
            .bind(&prompt.prompt_id)
            .bind(&scroll.scroll_id)
            .execute(pool)
            .await
            .unwrap();
    }

    let prompts: Vec<Prompt> = sqlx::query_as::<_, Prompt>(
        "SELECT * 
        FROM prompts 
        WHERE scroll_id = $1;") // Find last prompt
        .bind(&prompt.scroll_id)
        .fetch_all(pool)
        .await
        .unwrap();

    for (idx, row) in prompts.iter().enumerate() {
        println!(" [{}]: content {}, curr_id {}, next_id {} \n", idx, row.content, row.prompt_id, row.next_prompt_id);
    }

    if let Some(last_prompt) = prompts.iter().find(|p| p.next_prompt_id == prompt.next_prompt_id) {
        sqlx::query(
            "UPDATE prompts 
            SET next_prompt_id = $1 
            WHERE prompt_id = $2")
            .bind(&prompt.prompt_id)
            .bind(&last_prompt.prompt_id)
            .execute(pool)
            .await
            .unwrap();

    } else {
        println!("No matching prompt found.");
    }

    Ok(())
}

pub async fn update_prompt(pool: &SqlitePool, prompt: &Prompt, answer: &str) -> Result<()>{

    sqlx::query(
        "UPDATE prompts 
        SET output = $1 
        WHERE prompt_id = $2")
        .bind(&answer)
        .bind(&prompt.prompt_id)
        .execute(pool)
        .await
        .unwrap();

    Ok(())

}

// Sorted from first to last prompt on the list
pub async fn get_prompts_from_scroll(pool: &SqlitePool, scroll: &Scroll)-> Result<Vec<Prompt>> {
    let prompts: Vec<Prompt> = sqlx::query_as::<_, Prompt>(
        "SELECT * 
        FROM prompts
        WHERE scroll_id = $1;")
        .bind(&scroll.scroll_id)
        .fetch_all(pool)
        .await
        .unwrap();

    println!("Current prompts: \n");
    for (idx, row) in prompts.iter().enumerate() {
        println!(" [{}]: curr_id {}, next_id {} \n", idx, row.prompt_id, row.next_prompt_id);
    }

    Ok(prompts)
}
