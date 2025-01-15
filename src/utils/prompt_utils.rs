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
    let mut chain: Vec<Prompt> = vec![prompt.to_owned()];
    let mut prev_prompt_id = &prompt.prev_prompt_id;
    while prev_prompt_id != &prompt.project_id {
        let curr_prompt = prompts.iter().find(
            |p| &p.prompt_id == prev_prompt_id
        ).unwrap();
        chain.push(curr_prompt.to_owned());
        prev_prompt_id = &curr_prompt.prev_prompt_id;
    }
    chain.reverse();
    return chain
} 

pub fn format_prompt(p: &Prompt)-> String {
    format!(
        " |- Content: {} \n | Output: {}...",
        if p.content.chars().count() < 40 {
            &p.content
        } else {
            &p.content[0..20]
        },
        if p.output.chars().count() < 40 {
            &p.output
        } else {
            &p.output[0..20]
        }
    )
}

pub fn format_prompt_depth(prompt: &Prompt, b_depth: &str)-> String {
    format!(
        "{b_depth}- Content: {:?}... \n{b_depth}  Output: {:?}...",
        if prompt.content.chars().count() < 20 {
            &prompt.content
        } else {
            &prompt.content[0..20]
        },
        if prompt.output.chars().count() < 20 {
            &prompt.output
        } else {
            &prompt.output[0..20]
        },
    )
}
