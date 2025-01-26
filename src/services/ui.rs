use anyhow::{Context, Ok, Result};
use sqlx::SqlitePool;

use crate::{
    core::{
        prompt::{format_prompt, format_prompt_depth},
        scroll::get_scrolls,
    },
    utils::structs::{Project, Prompt},
};

pub async fn usr_scrolls(pool: &SqlitePool, project: &Project) -> Result<Vec<String>> {
    let scrolls = get_scrolls(pool, &project.project_id)
        .await
        .context("Failed to fetch scrolls from the database")?;

    Ok(scrolls
        .into_iter()
        .filter_map(|row| row.scroll_path.split("/").last().map(|s| s.to_string()))
        .collect())
}

pub fn helper_print(prompts: &Vec<Prompt>, prompt: &Prompt, b_depth: &str) -> Result<Vec<String>> {
    let mut format_vec: Vec<String> = vec![];
    format_vec.push(b_depth.to_string());

    let (p_str, o_str) = format_prompt_depth(prompt, b_depth);
    format_vec.push(p_str);
    format_vec.push(o_str);

    let new_indent = format!("{}  |", b_depth); // Append to the current indentation for children

    let child_prompts: Vec<&Prompt> = prompts
        .iter()
        .filter(|p| p.prev_prompt_id == prompt.prompt_id)
        .collect();

    for p in child_prompts.iter() {
        let child_vec = helper_print(prompts, p, &new_indent)
            .with_context(|| format!("Error processing child prompt with ID: {:?}", p.prompt_id))?;
        format_vec.extend(child_vec);
    }

    Ok(format_vec)
}

pub async fn usr_prompts(prompts: &Vec<Prompt>) -> Result<Vec<String>> {
    let fst_prompts: Vec<&Prompt> = prompts
        .iter()
        .filter(|p| p.prev_prompt_id == p.project_id)
        .collect();

    let mut format_vec: Vec<String> = vec![];
    for fst_prompt in fst_prompts.iter() {
        let child_vec = helper_print(prompts, fst_prompt, "  |").with_context(|| {
            format!(
                "Error processing first prompt with ID: {:?}",
                fst_prompt.prompt_id
            )
        })?;
        format_vec.extend(child_vec);
    }

    Ok(format_vec)
}

pub fn usr_prompt_chain(prompts: &[Prompt]) -> Vec<String> {
    let mut str_items: Vec<String> = Vec::new();
    for p in prompts.iter() {
        let (p_str, o_str) = format_prompt(p);
        // Reverse order for fst at top
        str_items.push(o_str);
        str_items.push(p_str);
    }
    str_items.reverse();
    str_items
}
