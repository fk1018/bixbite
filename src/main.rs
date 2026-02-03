use anyhow::Result;
use clap::{Parser, Subcommand};

use bixbite::commands::{build, check};

#[derive(Debug, Parser)]
#[command(name = "bixbite")]
#[command(about = "Bixbite compiler CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Build,
    Check {
        #[arg(long)]
        sorbet: bool,
        #[arg(long, value_enum, default_value_t = check::OutputFormat::Human)]
        format: check::OutputFormat,
    },
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Build => build::run(),
        Command::Check { sorbet, format } => check::run(check::CheckOptions { sorbet, format }),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
