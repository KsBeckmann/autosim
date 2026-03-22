use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(name = "autosim")]
#[command(about = "Um compilador para a linguagem AutoSim")]
#[command(version)]
#[command(author)]
#[command(disable_help_flag = true)]
#[command(disable_version_flag = true)]
#[command(help_template = "\
{about}

Uso: {usage}

Opções:
{options}")]
pub struct Args {
    /// Caminho para o arquivo fonte a ser compilado
    #[arg(short, long, required = true)]
    pub path: PathBuf,

    /// Exibe ajuda
    #[arg(short, long, action = clap::ArgAction::Help)]
    help: Option<bool>,

    /// Exibe a versão
    #[arg(short = 'V', long, action = clap::ArgAction::Version)]
    version: Option<bool>,
}
