use std::{borrow::Cow, path::PathBuf, thread::spawn};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ignore::{DirEntry, WalkBuilder, WalkState};
use nucleo_picker::{nucleo::Config, render::StrRenderer, PickerOptions, Render};

use crate::utils::error::{AppError, Result as AppResult};

pub struct DirEntryRender;

impl Render<DirEntry> for DirEntryRender {
    type Str<'a> = Cow<'a, str>;

    /// Render a `DirEntry` using its internal path buffer.
    fn render<'a>(&self, value: &'a DirEntry) -> Self::Str<'a> {
        value.path().to_string_lossy()
    }
}

pub fn select_files(dir_path: Option<&str>) -> AppResult<Option<String>> {
    disable_raw_mode()
        .map_err(|_| AppError::UnexpectedError("Failed to disable raw mode".into()))?;
    let mut picker = PickerOptions::default()
        // See the nucleo configuration for more options:
        //   https://docs.rs/nucleo/latest/nucleo/struct.Config.html
        .config(Config::DEFAULT.match_paths())
        // Use our custom renderer for a `DirEntry`
        .picker(DirEntryRender);

    // "argument parsing"
    let root: PathBuf = dir_path.map(Into::into).unwrap_or_else(|| "/home/".into());

    // populate from a separate thread to avoid locking the picker interface
    let injector = picker.injector();
    spawn(move || {
        WalkBuilder::new(root).build_parallel().run(|| {
            let injector = injector.clone();
            Box::new(move |walk_res| {
                match walk_res {
                    Ok(dir) => injector.push(dir),
                    Err(err) => {
                        // Log the error or handle it appropriately
                        eprintln!("Error during directory walk: {}", err);
                    }
                }
                WalkState::Continue
            })
        });
    });

    let file: Option<String> = picker
        .pick()
        .map_err(|_| AppError::UnexpectedError("Picker failed to pick a file".into()))?
        .map(|entry| entry.path().display().to_string());

    enable_raw_mode().map_err(|_| AppError::UnexpectedError("Failed to enable raw mode".into()))?;
    Ok(file)
}

pub fn item_selector(items: Vec<String>) -> AppResult<Option<String>> {
    disable_raw_mode()
        .map_err(|_| AppError::UnexpectedError("Failed to disable raw mode".into()))?;

    let mut picker = PickerOptions::default()
        .config(Config::DEFAULT.match_paths())
        .picker(StrRenderer);

    let injector = picker.injector();
    for item in items {
        injector.push(item);
    }

    let sel_item: Option<String> = picker
        .pick()
        .map_err(|_| AppError::UnexpectedError("Picker failed to pick an item".into()))?
        .map(|opt| opt.to_string());

    enable_raw_mode().map_err(|_| AppError::UnexpectedError("Failed to enable raw mode".into()))?;

    Ok(sel_item)
}
