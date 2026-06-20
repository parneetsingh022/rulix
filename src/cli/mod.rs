mod list;

use std::path::PathBuf;

use crate::rules::{RulesFileSource, default_rules_file};
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the config file.
    #[arg(short, long, global = true)]
    rules: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display all configured rules.
    List,

    /// Run configured rules.
    Run,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        let rules_path = match &self.rules {
            Some(path) => RulesFileSource::User(path.clone()),
            None => RulesFileSource::Default(default_rules_file()),
        };

        match &self.command {
            Commands::List => list::run(rules_path)?,

            Commands::Run => {}
        }

        Ok(())
    }
}
