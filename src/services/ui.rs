use std::io;
use std::io::Write;

use anyhow::{Ok, Result};
use sqlx::sqlite::SqliteRow;

pub fn usr_ask(sel: &String)-> Result<usize> {
    println!("{}", sel);
    io::stdout().flush()?;

    let mut ans = String::new();
    io::stdin().read_line(&mut ans)
        .ok()
        .expect("Failed to read line");
    Ok(ans.trim().parse::<usize>().unwrap())
}

