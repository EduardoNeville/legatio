use std::io;
use std::io::Write;
use anyhow::{Ok, Result};

use sqlx::SqlitePool;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

use crate::db::file::get_files;
use crate::db::prompt::get_prompts;
use crate::utils::structs::{ Project, Prompt };

pub fn usr_ask(sel: &str)-> Result<usize> {
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

async fn helper_print(prompts: &Vec<Prompt>, prompt: &Prompt, depth: &usize)-> Result<()> {
    let child_prompts: Vec<&Prompt> = prompts.iter().filter(
        |p| &p.prev_prompt_id == &prompt.prompt_id
    ).collect();
    
    let b_depth = "| ".repeat(*depth);
    println!("{:?}", b_depth);
    println!(
        "{:?}- [{:?}] Content: {:?} \n{:?}  Output: {:?}",
        b_depth,
        prompt.idx,
        prompt.content.get(0..20),
        b_depth,
        prompt.output.get(0..20)
    );

    let new_depth = depth + 1;
    if child_prompts.len() != 0 {
        child_prompts.iter().map(|p| helper_print(prompts, &p, &new_depth));
    }

    Ok(())
}

pub async fn usr_prompts(pool: &SqlitePool, project_id: &str) -> Result<()> {
    let prompts = get_prompts(pool, &project_id).await.unwrap();

    let depth = 0;
    let fst_prompts: Vec<&Prompt> = prompts.iter().filter(
        |p| &p.prev_prompt_id == &project_id
    ).collect();
    
    let idx = 0;
    for fst_prompt in fst_prompts.iter() {
        helper_print(&prompts, fst_prompt, &depth).await.unwrap();
    }

    Ok(())
}
