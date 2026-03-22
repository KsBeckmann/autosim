use std::fs;
use anyhow::anyhow;
use clap::Parser;
use logos::Logos;

use autosim::cli::Args;
use autosim::lexer::Token;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let file_content = fs::read_to_string(args.path)?;

    let lex = Token::lexer(&file_content);

    for token in lex {
        let token = token.map_err(|_| anyhow!("Erro ao processar token"))?;
        println!("{:?}", token);
    }

    Ok(())
}
