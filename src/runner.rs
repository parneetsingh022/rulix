//! Executes rules and steps defined in a [`RuleSet`].
//!
//! A `Runner` processes each rule in sequence and executes all associated
//! steps against the rule's target. State produced during execution, such as
//! files matched by `Match` steps, is accumulated in-memory and shared across
//! subsequent steps within the same run.
//!
//! Steps are responsible for implementing their own behavior, while the
//! `Runner` coordinates execution order and manages shared execution state.

use anyhow::Result;
use std::path::PathBuf;

use crate::rules::{Rule, RuleSet};

pub struct Runner {
    rule_set: RuleSet,
    matched_files: Vec<PathBuf>,
}

impl Runner {
    pub fn new(rule_set: RuleSet) -> Self {
        Self {
            rule_set,
            matched_files: Vec::new(),
        }
    }

    /// Executes all rules in the configured rule set.
    ///
    /// Execution stops on the first error.
    pub fn run(&mut self) -> Result<()> {
        let rule_set = &self.rule_set;
        let matched_files = &mut self.matched_files;

        for rule in rule_set {
            Self::run_steps(rule, matched_files)?;
        }

        Ok(())
    }

    fn run_steps(rule: &Rule, matched_files: &mut Vec<PathBuf>) -> Result<()> {
        for step in &rule.steps {
            step.execute(&rule.target, matched_files)?;
        }

        Ok(())
    }
}
