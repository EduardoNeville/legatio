use std::io;
use std::io::Write;
use anyhow::{Ok, Result};

use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use sqlx::SqlitePool;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

use crate::db::scroll::get_scrolls;
use crate::db::prompt::get_prompts;
use crate::utils::prompt_utils::{format_prompt, format_prompt_depth};
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
    let theme = ts.themes["base16-ocean.dark"].clone();

    let parser = Parser::new(s);

    let mut syntax = ps.find_syntax_by_extension(extension).unwrap();
    let mut code = String::new();
    let mut in_code_block = false;

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                let lang = lang.trim();
                syntax = ps
                    .find_syntax_by_token(lang)
                    .unwrap_or_else(|| ps.find_syntax_plain_text());
                in_code_block = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                if in_code_block {
                    let mut highlighter = HighlightLines::new(&syntax, &theme);
                    for line in LinesWithEndings::from(&code) {
                        let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &ps).unwrap();
                        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
                        print!("{}", escaped);
                    }
                    code.clear();
                    in_code_block = false;
                }
            }
            Event::Text(text) => {
                if in_code_block {
                    code.push_str(&text);
                } else {
                    let mut highlighter = HighlightLines::new(&syntax, &theme);
                    for line in LinesWithEndings::from(&text) {
                        let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &ps).unwrap();
                        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
                        print!("{}", escaped);
                    }
                }
            }
            _ => {}
        }
    }
}

pub async fn usr_scrolls(pool: &SqlitePool, project: &Project) -> Result<()> {
    let scrolls = get_scrolls(pool, &project.project_id).await.unwrap();
    println!("Current scrolls: \n");
    for (idx, row) in scrolls.iter().enumerate() {
        let scrollname = row.scroll_path.split("/").last().unwrap();
        println!(" [{}]: {} \n", idx, scrollname);
        //highlight(&row.content.get(0..50).unwrap(), scrollname.split(".").last().unwrap()); 
    }

    Ok(())
}

async fn helper_print(prompts: &Vec<Prompt>, prompt: &Prompt, depth: &usize)-> Result<()> {
    let child_prompts: Vec<&Prompt> = prompts.iter().filter(
        |p| &p.prev_prompt_id == &prompt.prompt_id
    ).collect();
    
    let b_depth = "  |";
    let _ = b_depth.repeat(*depth);
    println!("{b_depth}");
    println!("{}", format_prompt_depth(prompt, b_depth));

    let new_depth = depth + 1;
    if child_prompts.len() != 0 {
        let _ = child_prompts.iter().map(|p| helper_print(prompts, &p, &new_depth));
    }

    Ok(())
}

pub async fn usr_prompts(pool: &SqlitePool, project_id: &str) -> Result<()> {
    let prompts = get_prompts(pool, &project_id).await.unwrap();

    let depth = 0;
    let fst_prompts: Vec<&Prompt> = prompts.iter().filter(
        |p| &p.prev_prompt_id == &project_id
    ).collect();
    
    for fst_prompt in fst_prompts.iter() {
        helper_print(&prompts, fst_prompt, &depth).await.unwrap();
    }

    Ok(())
}

pub fn clear_screen() {
    // Print the escape sequence to clear the terminal
    print!("\x1B[2J\x1B[H");
    io::stdout().flush().unwrap();
}

pub fn usr_prompt_chain(prompts: &[Prompt]) -> Result<()> {
    let _ = prompts.iter().map(|p| {
        println!("{}", format_prompt(p));
    });

    Ok(())
}

