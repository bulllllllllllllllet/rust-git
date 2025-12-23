use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;
mod objects;
mod utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Add {
        path: String,
    },
    Commit {
        #[arg(short, long)]
        message: String,
    },
    Rm {
        path: String,
    },
    Branch {
        name: String,
    },
    Checkout {
        name: String,
    },
    Merge {
        branch: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => commands::init(),
        Commands::Add { path } => commands::add(path),
        Commands::Commit { message } => commands::commit(message),
        Commands::Rm { path } => commands::rm(path),
        Commands::Branch { name } => commands::branch(name),
        Commands::Checkout { name } => commands::checkout(name),
        Commands::Merge { branch } => commands::merge(branch),
    }
}
