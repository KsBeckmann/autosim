use std::path::Path;
use std::{fs, process};

use autosim::cli::Args;
use autosim::parser::Program;
use autosim::{lexer, parser, sema, ui};
use clap::Parser;

fn main() -> iced::Result {
    let args = Args::parse();

    let Some(program) = load_program(&args.path) else {
        process::exit(1);
    };

    iced::application(
        move || ui::State::new(program.clone()),
        ui::update,
        ui::view,
    )
    .subscription(ui::subscription)
    .title("autosim")
    .run()
}

/// Lê e valida o arquivo-fonte, imprimindo no stderr qualquer erro encontrado.
fn load_program(path: &Path) -> Option<Program> {
    let path_str = path.display().to_string();

    let source = fs::read_to_string(path)
        .inspect_err(|e| eprintln!("erro ao ler {path_str}: {e}"))
        .ok()?;

    let spanned = lexer::tokenize_spanned(&source)
        .inspect_err(|e| e.report(&path_str, &source))
        .ok()?;
    let (tokens, spans): (Vec<_>, Vec<_>) = spanned.into_iter().unzip();

    let program = parser::parse(&tokens, &spans)
        .inspect_err(|errors| errors.iter().for_each(|e| e.report(&path_str, &source)))
        .ok()?;

    sema::analyse(&program, &spans)
        .inspect_err(|errors| errors.iter().for_each(|e| e.report(&path_str, &source)))
        .ok()?;

    if program.simulations.is_empty() {
        eprintln!(
            "nenhuma declaração `simular` encontrada em {path_str}. \
             adicione ao menos uma simulação para visualizar a execução."
        );
        return None;
    }

    Some(program)
}
