#![allow(unused)]

use std::io::{BufRead, BufReader};
use clap::Parser;
use anyhow::{Context, Result};


/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    pattern: String,
    /// The path to the file to read
    path: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    println!("pattern is {pattern} and path is {path:?}",
             pattern = args.pattern,
             path = args.path);

    let path = &args.path;

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("could not read file `{}`", path.display()))?;

    for line in content.lines() {
        if line.contains(&args.pattern) {
            println!("{}", line);
        }
    }
    Ok(())
}
