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
