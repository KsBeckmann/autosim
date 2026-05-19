use clap::Parser;
use std::{fs, process};

use autosim::cli::Args;
use autosim::ui;

fn main() -> iced::Result {
    let args = Args::parse();

    let path_str = args.path.display().to_string();
    let file_content = match fs::read_to_string(&args.path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("erro ao ler {path_str}: {e}");
            process::exit(1);
        }
    };

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

    if let Err(errors) = autosim::sema::analyse(&program, &spans) {
        for err in &errors {
            err.report(&path_str, &file_content);
        }
        process::exit(1);
    }

    if program.simulations.is_empty() {
        eprintln!(
            "nenhuma declaração `simular` encontrada em {path_str}. \
             adicione ao menos uma simulação para visualizar a execução."
        );
        process::exit(1);
    }

    iced::application(
        move || ui::State::new(program.clone()),
        ui::update,
        ui::view,
    )
    .subscription(ui::subscription)
    .title("autosim")
    .run()
}
