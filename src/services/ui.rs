use std::fs;

use anyhow::{Context, Ok, Result};
use ratatui::style::Color;
use sqlx::SqlitePool;

use crate::{
    core::{
        prompt::{format_prompt, format_prompt_depth},
        scroll::get_scrolls,
    },
    utils::{
        error::AppError,
        logger::log_error,
        structs::{Project, Prompt},
    },
};

use toml::Value;

use super::config::get_config_dir;

pub struct ThemeColors {
    pub background: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
}

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

pub fn extract_theme_colors(theme_name: &str) -> Result<ThemeColors> {
    let config_dir = get_config_dir()?;
    let themes_path = config_dir.join("themes.toml");

    // Read the themes.toml file
    let themes_content = fs::read_to_string(&themes_path).map_err(|e| {
        log_error(&format!(
            "Failed to read file {}: {}",
            themes_path.to_string_lossy(),
            e
        ));
        AppError::FileError(format!(
            "Failed to read file {}: {}",
            themes_path.to_string_lossy(),
            e
        ))
    })?;

    // Parse TOML file into a generic Value
    let themes_toml: Value = themes_content.parse::<Value>().map_err(|e| {
        log_error(&format!("Failed to parse TOML: {}", e));
        AppError::ParseError(format!("Failed to parse TOML: {}", e))
    })?;

    // Locate the `themes` array in the TOML file
    let default = Color::Rgb(0, 0, 0);
    if let Some(themes) = themes_toml.get("themes").and_then(|t| t.as_array()) {
        // Search for the theme with matching name
        for theme in themes {
            if let Some(name) = theme.get("name").and_then(|n| n.as_str()) {
                if name == theme_name {
                    return Ok(ThemeColors {
                        background: hex_to_tui_color(
                            theme
                                .get("background")
                                .and_then(|b| b.as_str())
                                .unwrap_or("#"),
                        )
                        .unwrap_or(default),
                        primary: hex_to_tui_color(
                            theme.get("primary").and_then(|p| p.as_str()).unwrap_or("#"),
                        )
                        .unwrap_or(default),
                        secondary: hex_to_tui_color(
                            theme
                                .get("secondary")
                                .and_then(|s| s.as_str())
                                .unwrap_or("#"),
                        )
                        .unwrap_or(default),
                        accent: hex_to_tui_color(
                            theme.get("accent").and_then(|a| a.as_str()).unwrap_or("#"),
                        )
                        .unwrap_or(default),
                    });
                }
            }
        }
    }

    Ok(ThemeColors {
        background: default,
        primary: default,
        secondary: default,
        accent: default,
    })
}

// Converts an RGB tuple to `ratatui::Color`
fn hex_to_tui_color(hex: &str) -> Result<Color> {
    let hex = hex.trim_start_matches('#');

    let mut r: u8 = 0;
    let mut g: u8 = 0;
    let mut b: u8 = 0;
    let hex_len = hex.len();
    if hex_len >= 2 {
        r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|err| {
                log_error(&format!(
                    "Invalid red component in: {}, error: {}",
                    hex, err
                ));
                AppError::ParseError(format!(
                    "Invalid red component in: {}. Reason: {}",
                    hex, err
                ));
            })
            .unwrap();
    }
    if hex_len >= 4 {
        g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|err| {
                log_error(&format!(
                    "Invalid green component in: {}, error: {}",
                    hex, err
                ));
                AppError::ParseError(format!(
                    "Invalid green component in: {}. Reason: {}",
                    hex, err
                ));
            })
            .unwrap();
    }
    if hex_len == 6 {
        b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|err| {
                log_error(&format!(
                    "Invalid blue component in: {}, error: {}",
                    hex, err
                ));
                AppError::ParseError(format!(
                    "Invalid blue component in: {}. Reason: {}",
                    hex, err
                ));
            })
            .unwrap();
    }

    Ok(Color::Rgb(r, g, b))
}
