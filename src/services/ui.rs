use std::io;
use std::io::Write;
use anyhow::{Ok, Result};

use sqlx::SqlitePool;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

use crate::db::file::get_files;
use crate::db::prompt::get_prompts_from_scroll;
use crate::utils::structs::{Project, Scroll};

pub fn usr_ask(sel: &String)-> Result<usize> {
    println!("{}", sel);
    io::stdout().flush()?;

    let mut ans = String::new();
    io::stdin().read_line(&mut ans)
        .ok()
        .expect("Failed to read line");
    Ok(ans.trim().parse::<usize>().unwrap())
}

pub fn highlight(s: &str, extension: &str) {
    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    println!("extension: {}", &extension);

    let syntax = ps.find_syntax_by_extension(&extension).unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

    for line in LinesWithEndings::from(s) { // LinesWithEndings enables use of newlines mode
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        println!("{}", escaped);
    }
}

pub async fn usr_files(pool: &SqlitePool, project: &Project) -> Result<()> {
    let files = get_files(pool, &project.project_id).await.unwrap();
    println!("Current files: \n");
    for (idx, row) in files.iter().enumerate() {
        let filename = row.file_path.split("/").last().unwrap();
        println!(" [{}]: {} \n", idx, filename);
        highlight(&row.content, filename.split(".").last().unwrap()); 
    }

    Ok(())
}

pub async fn usr_prompts(pool: &SqlitePool, scroll: &Scroll) -> Result<()> {
    let prompts = get_prompts_from_scroll(pool, &scroll.scroll_id).await.unwrap();
    let curr_prompt = prompts.iter().find(|p| p.prompt_id == scroll.init_prompt_id).unwrap();

    let mut counter = 0;
    while curr_prompt.next_prompt_id != "" {
        println!("[{}] Prompt - Content", counter);
        highlight(&curr_prompt.content, "md");
        println!("[{}] Prompt - Answer", counter);
        highlight(&curr_prompt.output, "md");
        counter = counter + 1;
    }

    Ok(())
}
