#![allow(unused)]

use std::io::{BufRead, BufReader};
use clap::Parser;


/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    pattern: String,
    /// The path to the file to read
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();

    println!("pattern is {pattern} and path is {path:?}",
             pattern = args.pattern,
             path = args.path);

    let mut reader = BufReader::new(std::fs::File::open(&args.path).expect("could not open file"));
    let mut line = String::new();

    loop {
        let len = reader.read_line(&mut line).expect("could not read line");
        if len == 0 {
            break;
        }
        if line.contains(&args.pattern) {
            print!("{}", line);
        }
        line.truncate(0);
    }
}
