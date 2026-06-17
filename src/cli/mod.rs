mod list;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the config file.
    #[arg(short, long, default_value = "local/config.yaml")]
    config: PathBuf,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display all configured rules.
    List,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match self.command {
            Commands::List => list::run(&self.config)?,
        }

        Ok(())
    }
}
