use std::collections::HashMap;

use super::structs::{Scroll, Prompt};

pub fn system_prompt(scrolls: &[Scroll])-> String {
    let system_prompt = scrolls.iter()
        .map(|scroll| {
            let scroll_name = scroll.scroll_path.rsplit('/').next().unwrap_or(""); // Handles empty paths safely
            format!("```{:?}\n{:?}```\n", scroll_name, scroll.content)
        })
        .collect::<Vec<_>>()
        .join(""); // Joining avoids intermediate allocations with push_str
    
    return system_prompt
}

pub fn prompt_chain(prompts: &[Prompt], prompt: &Prompt) -> Vec<Prompt> {
    let mut prompt_map: HashMap<&str, &Prompt> = prompts
        .into_iter()
        .map(|prompt| (prompt.prompt_id.as_ref(), prompt))
        .collect();

    let mut chain = Vec::<Prompt>::new();
    let mut current_id: Option<&str> = Some(prompt.prompt_id.as_ref());

    while let Some(id) = current_id {
        if let Some(prompt) = prompt_map.remove(id) {
            current_id = if prompt.prev_prompt_id.is_empty() {
                None
            } else {
                Some(prompt.prev_prompt_id.as_ref())
            };
            chain.push(prompt.to_owned());
        } else {
            break;
        }
    }

    return chain;
}

pub fn format_prompt(p: &Prompt)-> (String, String) {
    let p_str = format!(" |- Prompt: {:?} ",
        if p.content.chars().count() < 40 {
            p.content.replace('\n', " ").to_string()
        } else {
            p.content[0..40].replace('\n', " ").to_string()
        },
    );
    let o_str = format!(" |- Output: {:?}",
        if p.output.chars().count() < 40 {
            p.output.replace('\n', " ").to_string()
        } else {
            p.output[0..40].replace('\n', " ").to_string()
        }
    );

    (p_str, o_str)
}

pub fn format_prompt_depth(p: &Prompt, b_depth: &str)-> (String, String) {
    let p_str = format!("{b_depth}> Prompt: {:?} ",
        if p.content.chars().count() < 40 {
            p.content.replace('\n', " ").to_string()
        } else {
            p.content[0..40].replace('\n', " ").to_string()
        },
    );
    let o_str = format!("{b_depth}> Output: {:?}",
        if p.output.chars().count() < 40 {
            p.output.replace('\n', " ").to_string()
        } else {
            p.output[0..40].replace('\n', " ").to_string()
        }
    );

    (p_str, o_str)
}
