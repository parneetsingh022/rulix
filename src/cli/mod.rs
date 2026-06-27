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

    /// Execute Configured rules.
    ///
    /// By default, this performs a dry run and previews the file system changes
    /// that would be made. Use `--execute` to actually make those changes.
    Run {
        /// Apply filesystem changes instead of previewing them.
        #[arg(long)]
        execute: bool,
    },
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

            Commands::Run { execute } => {
                let dry_run = !execute;

                let mut runner = Runner::new(rules, dry_run);
                runner.run()?;
            }
        }

        Ok(())
    }
}
