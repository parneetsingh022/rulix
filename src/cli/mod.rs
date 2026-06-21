mod list;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::{
    errors::FileError,
    rules::{RuleSet, RulesFileSource, default_rules_file},
    runner::Runner,
};

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
        let source = match &self.rules {
            Some(path) => RulesFileSource::User(path.clone()),
            None => RulesFileSource::Default(default_rules_file()),
        };

        let rules = match RuleSet::from_file(source.path()) {
            Ok(rules) => rules,

            // When `--rules` is not provided, Rulix falls back to its default rules file.
            // That file may not exist yet, especially on first startup, so a missing
            // default rules file is not treated as an error.
            //
            // If the user explicitly provides a path with `--rules`, then that file is
            // expected to exist and a missing file should be reported as an error.
            Err(FileError::NotFound(_)) if source.is_default() => {
                println!("No rules to show.");
                return Ok(());
            }

            Err(err) => return Err(err.into()),
        };

        match &self.command {
            Commands::List => list::run(rules)?,

            Commands::Run => {
                let mut runner = Runner::new(rules);
                runner.run()?;
            }
        }

        Ok(())
    }
}
