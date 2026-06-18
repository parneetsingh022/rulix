mod list;

use std::{
    path::{PathBuf},
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use crate::rules::{default_rules_file, RulesSource};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the config file.
    #[arg(short, long, global=true)]
    rules: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display all configured rules.
    List,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Commands::List => {
                let rules_path = match &self.rules {
                    Some(path) => RulesSource::User(path.clone()),
                    None => RulesSource::Default(default_rules_file()),
                };

                list::run(rules_path)?
            },
        }

        Ok(())
    }
}
