use nucleo::*;
use anyhow::Result;

pub fn select_files(file_list: Vec<String>, multi: bool) -> Result<Vec<usize>> {
    // Join file list into a single string, where each file is on a new line
    let input = file_list.join("\n");

    // Create a query matcher using `nucleo`
    let mut query_engine = QueryEngine::new()
        .case_insensitive(true) // Enable case-insensitive matching
        .multiple_selections(multi); // Allow multiple selections if `multi` is true

    // Populate the query engine with entries from the file list
    query_engine.add_values(
        file_list
            .into_iter()
            .enumerate()
            .map(|(index, value)| (index, value)),
    );

    // Run the query engine to process user input
    let selected_indices = query_engine.run(|state| {
        // Optional: Display the search prompt and real-time results
        let prompt = if multi { "Multi-select files:" } else { "Select a file:" };
        state.display_prompt(prompt); // Customize the prompt message
    })?;

    // Extract selected indices
    let sel_idxs = selected_indices.into_iter().collect::<Vec<usize>>();

    Ok(sel_idxs)
}
