use skim::prelude::*;
use anyhow::Result;
use std::io::Cursor;

pub fn select_files(file_list: Vec<String>) -> Result<Vec<String>> {
    let options = SkimOptionsBuilder::default()
        .multi(true)
        .bind(vec!["change:reload(cat {})"])
        .build()
        .unwrap();

    let item_reader = SkimItemReader::default();

    // Join the file list into a single string
    let input = file_list.join("\n");

    // Convert the string into a Vec<u8>, so we own the data
    let bytes = input.into_bytes();

    // Create a Cursor over the Vec<u8>, which implements BufRead + Send + 'static
    let cursor = Cursor::new(bytes);

    // Pass the Cursor to of_bufread
    let items = item_reader.of_bufread(cursor);

    let selected_items = Skim::run_with(&options, Some(items))
        .map(|out| {
            out.selected_items
                .iter()
                .map(|item| item.output().to_string())
                .collect()
        })
        .unwrap_or_else(Vec::new);

    Ok(selected_items)
}
