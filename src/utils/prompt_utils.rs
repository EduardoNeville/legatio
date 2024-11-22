
pub fn construct_system_prompt(files: &[(String, String)]) -> String {
    let mut system_prompt = String::new();
    for (_path, content) in files {
        system_prompt.push_str(content);
        system_prompt.push_str("\n");
    }
    system_prompt
}

