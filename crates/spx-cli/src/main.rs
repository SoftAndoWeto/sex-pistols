mod cli;
mod error;
mod graph;
mod node;

#[cfg(test)]
mod tests;

use clap::Parser;
use miette::IntoDiagnostic;

use cli::{Cli, Command};

fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run { file, args } => {
            let output = graph::build(&file).into_diagnostic()?;
            let loader_dir = node::loader_dir();
            let status = node::run(&file, &args, output.cache.l2_dir(), &loader_dir)
                .into_diagnostic()?;
            std::process::exit(status.code().unwrap_or(1));
        }
        Command::Watch { .. } => {
            eprintln!("spx watch: coming soon");
        }
        Command::Repl => {
            eprintln!("spx repl: coming soon");
        }
    }

    Ok(())
}
