use std::{fs, process};
use clap::Parser;

use autosim::cli::Args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let path_str = args.path.display().to_string();
    let file_content = fs::read_to_string(&args.path)?;

    let spanned = match autosim::lexer::tokenize_spanned(&file_content) {
        Ok(v) => v,
        Err(e) => {
            e.report(&path_str, &file_content);
            process::exit(1);
        }
    };

    let (tokens, spans): (Vec<_>, Vec<_>) = spanned.into_iter().unzip();

    let program = match autosim::parser::parse(&tokens, &spans) {
        Ok(p) => p,
        Err(errors) => {
            for err in &errors {
                err.report(&path_str, &file_content);
            }
            process::exit(1);
        }
    };

    println!("{:#?}", program);

    Ok(())
}
