use skim::prelude::*;
use anyhow::Result;

pub fn select_files(file_list: &[String]) -> Result<Vec<String>> {
    let options = SkimOptionsBuilder::default()
        .multi(true)
        .build()
        .unwrap();

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(file_list.join("\n").as_bytes());

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
