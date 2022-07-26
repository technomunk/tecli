//! Technomunk's random CLI-based stuff

#![warn(clippy::all)]

mod img;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(subcommand)]
    Img(img::Command),
}

fn main() {
    // Parse empty args, as the user may have supplied a help flag
    let args = Cli::parse();

    match args.command {
        Commands::Img(img) => img::command(img),
    }
}
