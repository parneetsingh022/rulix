mod cli;
mod config;
mod errors;

use anyhow::Result;
use clap::Parser;
use config::RulixConfig;

use cli::Cli;

fn main() -> Result<()> {
    let args = Cli::parse();
    args.run()?;

    Ok(())
}
