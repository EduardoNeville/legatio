use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};
use anyhow::Result;

use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

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

//pub async fn highlight_text(file_path: &str, text: &str)-> Result<Vec<&Vec<String>>> {
//    // Load these once at the start of your program
//    let ps = SyntaxSet::load_defaults_newlines();
//    let ts = ThemeSet::load_defaults();
//
//    let extension = if file_path != "" { file_path.split(".").last().expect("No Text path") } else {"md"};
//    let syntax = ps.find_syntax_by_extension(extension).unwrap();
//    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
//
//    let mut out_str = vec![];
//    for line in LinesWithEndings::from(text) { // LinesWithEndings enables use of newlines mode
//        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
//        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
//        out_str.push(ranges);
//    }
//    Ok(out_str)
//}
