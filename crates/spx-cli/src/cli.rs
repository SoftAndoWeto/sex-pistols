use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "spx", about = "Fast TypeScript/JavaScript runner")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run a TypeScript or JavaScript file
    Run {
        /// Entry file
        file: PathBuf,
        /// Arguments forwarded to the script
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Watch a file and re-run on changes (coming soon)
    Watch {
        /// Entry file
        file: PathBuf,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Start an interactive REPL (coming soon)
    Repl,
}
