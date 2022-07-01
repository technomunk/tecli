//! Technomunk's random CLI-based stuff

#![warn(clippy::all)]

use clap::Parser;

/// Trivial CLI command that just prints hello world
#[derive(Parser, Debug)]
struct Args {}

fn main() {
    // Parse empty args, as the user may have supplied a help flag
    let _args = Args::parse();

    println!("Hello, world!");
}
