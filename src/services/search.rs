use anyhow::Result;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::Write;
use std::process::{Command, Stdio};
use walkdir::{DirEntry, WalkDir};

// directories (or files) to skip entirely
const IGNORE: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    "__pycache__",
    ".mypy_cache",
    ".ipynb_checkpoints",
];

fn is_not_ignored(entry: &DirEntry) -> bool {
    // get just the final component
    if let Some(name) = entry.file_name().to_str() {
        // always keep the root (it may be ".")
        if name.is_empty() {
            return true;
        }
        // skip if it matches any of our ignore-list
        !IGNORE.contains(&name)
    } else {
        true
    }
}

/// Spawn `fzf` on a filesystem crawl under `root`.
/// If `single_select == true`, we do *not* pass `--multi`
/// and we show a directory‐tree preview (`eza`).
/// If `single_select == false`, we *do* pass `--multi`
/// and we show a file preview (`bat`).
pub fn pick_paths(root: Option<&str>, single_select: bool) -> Result<Vec<String>> {
    // 1) drop into cooked mode for fzf
    disable_raw_mode()?;

    // 2) build our fzf invocation
    let mut cmd = Command::new("fzf");
    if !single_select {
        cmd.arg("--multi");
    }
    // choose preview pane based on single_select
    let preview = if single_select {
        // browse a tree when picking *dirs* (or single)
        "eza --icons --tree --level=2 --sort='size' --reverse \
         -a -I '.git|__pycache__|.mypy_cache|.ipynb_checkpoints|node_modules'"
    } else {
        // otherwise preview file contents
        "bat --style=numbers --color=always {}"
    };
    cmd.arg("--preview")
        .arg(preview)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    // 3) feed it all the file paths
    let mut child = cmd.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        let scan_root = root.unwrap_or(".");
        for entry in WalkDir::new(scan_root)
            // never descend into ignored dirs
            .into_iter()
            .filter_entry(is_not_ignored)
            // drop any Err
            .filter_map(Result::ok)
            // only files (you could also allow dirs if you want)
            .filter(|e| e.file_type().is_file())
        {
            writeln!(stdin, "{}", entry.path().display())?;
        }
        // dropping stdin closes the pipe
    }

    // 4) collect selections and restore raw‐mode
    let output = child.wait_with_output()?;
    enable_raw_mode()?;

    // 5) return each chosen line as a String
    let chosen = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect();
    Ok(chosen)
}

/// A generic helper to run `fzf` on an *in-memory* list of `items`.
/// `single_select == true` ⇒ no `--multi`;
/// `single_select == false` ⇒ `--multi`.
fn select_from_list(items: Vec<String>, single_select: bool) -> Result<Vec<String>> {
    disable_raw_mode()?;
    let mut cmd = Command::new("fzf");
    if !single_select {
        cmd.arg("--multi");
    }
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
    let mut child = cmd.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        for it in items {
            writeln!(stdin, "{}", it)?;
        }
    }
    let output = child.wait_with_output()?;
    enable_raw_mode()?;
    let chosen = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect();
    Ok(chosen)
}

/// Replace your old single‐pick.  Returns the *first* selected item if any.
pub fn item_selector(items: Vec<String>) -> Result<Option<String>> {
    let sel = select_from_list(items, true)?;
    Ok(sel.into_iter().next())
}

/// **New** multi‐pick helper.  Returns *all* selected items.
pub fn item_selector_multi(items: Vec<String>) -> Result<Vec<String>> {
    select_from_list(items, false)
}
