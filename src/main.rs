mod cli;
mod config;
mod errors;
mod rules;
mod runner;
mod steps;

use anyhow::Result;
use clap::Parser;

use cli::Cli;

fn main() -> Result<()> {
    let args = Cli::parse();
    args.run()?;

    Ok(())
}
