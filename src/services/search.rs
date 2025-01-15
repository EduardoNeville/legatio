use std::usize;
use std::{borrow::Cow, env::args, io, path::PathBuf, thread::spawn};

use std::io::Result;
use ignore::{DirEntry, WalkBuilder, WalkState};
use nucleo_picker::{nucleo::Config, PickerOptions, Render};
use nucleo_picker::{render::StrRenderer, Picker};

pub struct DirEntryRender;

impl Render<DirEntry> for DirEntryRender {
    type Str<'a> = Cow<'a, str>;

    /// Render a `DirEntry` using its internal path buffer.
    fn render<'a>(&self, value: &'a DirEntry) -> Self::Str<'a> {
        value.path().to_string_lossy()
    }
}

pub fn select_files(dir_path: Option<&str>) -> Result<String> {
    let mut picker = PickerOptions::default()
        // See the nucleo configuration for more options:
        //   https://docs.rs/nucleo/latest/nucleo/struct.Config.html
        .config(Config::DEFAULT.match_paths())
        // Use our custom renderer for a `DirEntry`
        .picker(DirEntryRender);

    // "argument parsing"
    let root: PathBuf = match dir_path {
        Some(path) => path.into(),
        None => "/home/".into() // TODO $HOME ??
    };

    // populate from a separate thread to avoid locking the picker interface
    let injector = picker.injector();
    spawn(move || {
        WalkBuilder::new(root).build_parallel().run(|| {
            let injector = injector.clone();
            Box::new(move |walk_res| {
                if let Ok(dir) = walk_res {
                    injector.push(dir);
                }
                WalkState::Continue
            })
        });
    });

    let file: Option<String> = match picker.pick()? {
        // the matched `entry` is `&DirEntry`
        Some(entry) => Some(entry.path().display().to_string()),
        None => None,
    };

    Ok(file.unwrap())
}

pub fn item_selector(items: Vec<String>) -> Result<Option<String>> {
    let mut picker = PickerOptions::default()
        // set the configuration to match 'path-like' objects
        .config(Config::DEFAULT.match_paths())
        .picker(StrRenderer);

    // populate the matcher
    let injector = picker.injector();
    for item in items {
        injector.push(item);
    }

    // open interactive prompt
    let sel_item: Option<String> = match picker.pick()? {
        Some(opt) => Some(opt.to_string()),
        None => None,
    };

    Ok(sel_item)
}

