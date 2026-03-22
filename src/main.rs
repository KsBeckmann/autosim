use std::{fs, process};
use clap::Parser;

use autosim::cli::Args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let path_str = args.path.display().to_string();
    let file_content = fs::read_to_string(&args.path)?;

    let tokens = match autosim::lexer::tokenize(&file_content, &path_str) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

    for token in &tokens {
        println!("{:?}", token);
    }

    Ok(())
}
