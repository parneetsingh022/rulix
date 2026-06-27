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

use console::style;

use crate::rules::{Rule, RuleSet};

pub struct Runner {
    rule_set: RuleSet,
    matched_files: Vec<PathBuf>,
    dry_run: bool,
}

impl Runner {
    pub fn new(rule_set: RuleSet, dry_run: bool) -> Self {
        Self {
            rule_set,
            matched_files: Vec::new(),
            dry_run,
        }
    }

    /// Executes all rules in the configured rule set.
    ///
    /// Execution stops on the first error.
    pub fn run(&mut self) -> Result<()> {
        let rule_set = &self.rule_set;
        let matched_files = &mut self.matched_files;
        let dry_run = self.dry_run;

        if dry_run {
            Self::print_dry_run_header();
        }

        for rule in rule_set {
            if dry_run {
                Self::print_dry_run_rule_header(rule);
            }

            Self::run_steps(rule, matched_files, dry_run)?;
        }

        if dry_run {
            Self::print_dry_run_footer();
        }

        Ok(())
    }

    fn run_steps(rule: &Rule, matched_files: &mut Vec<PathBuf>, dry_run: bool) -> Result<()> {
        for step in &rule.steps {
            if dry_run {
                step.dry_run(&rule.target, matched_files)?;
            } else {
                step.execute(&rule.target, matched_files)?;
            }
        }

        Ok(())
    }

    fn print_dry_run_header() {
        println!();
        println!("{}", style("DRY RUN").bold().cyan());
        println!("{}", style("Preview of planned file system changes").dim());
    }

    fn print_dry_run_rule_header(rule: &Rule) {
        println!();
        println!(
            "{} {}",
            style("rule").cyan().bold(),
            style(&rule.name).bold()
        );
        println!("{}", style("─".repeat(rule.name.len() + 5)).dim());
    }

    fn print_dry_run_footer() {
        println!(
            "{} No changes were made. Run again with {} to apply these changes.",
            style("note").yellow().bold(),
            style("--execute").cyan().bold()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{Rule, RuleSet};
    use crate::steps::Step;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn rule(name: &str, target: PathBuf, steps: Vec<Step>) -> Rule {
        Rule {
            name: name.to_string(),
            target,
            steps,
        }
    }

    fn rule_set(rules: Vec<Rule>) -> RuleSet {
        RuleSet {
            rules,
            path: PathBuf::from("test-rules.yaml"),
        }
    }

    #[test]
    fn run_executes_all_rules_and_last_match_replaces_previous_matches() {
        let dir1 = tempdir().unwrap();
        let dir2 = tempdir().unwrap();

        let file1 = dir1.path().join("a.txt");
        let file2 = dir2.path().join("b.txt");

        std::fs::write(&file1, "hello").unwrap();
        std::fs::write(&file2, "world").unwrap();

        let rule_set = rule_set(vec![
            rule(
                "first",
                dir1.path().to_path_buf(),
                vec![Step::new_match("txt")],
            ),
            rule(
                "second",
                dir2.path().to_path_buf(),
                vec![Step::new_match("txt")],
            ),
        ]);

        let mut runner = Runner::new(rule_set, false);

        runner.run().unwrap();

        assert_eq!(runner.matched_files, vec![file2]);
    }

    #[test]
    fn run_match_step_replaces_previous_matched_files() {
        let dir = tempdir().unwrap();

        let txt_file = dir.path().join("a.txt");
        let rs_file = dir.path().join("main.rs");

        std::fs::write(&txt_file, "hello").unwrap();
        std::fs::write(&rs_file, "fn main() {}").unwrap();

        let rule_set = rule_set(vec![rule(
            "match-multiple",
            dir.path().to_path_buf(),
            vec![Step::new_match("txt"), Step::new_match("rs")],
        )]);

        let mut runner = Runner::new(rule_set, false);

        runner.run().unwrap();

        assert_eq!(runner.matched_files, vec![rs_file]);
    }

    #[test]
    fn run_stops_on_first_error() {
        let dir = tempdir().unwrap();

        let missing_dir = dir.path().join("missing");
        let existing_dir = dir.path().join("existing");
        std::fs::create_dir(&existing_dir).unwrap();

        let file = existing_dir.join("should-not-match.txt");
        std::fs::write(&file, "hello").unwrap();

        let rule_set = rule_set(vec![
            rule("bad-rule", missing_dir, vec![Step::new_match("txt")]),
            rule("should-not-run", existing_dir, vec![Step::new_match("txt")]),
        ]);

        let mut runner = Runner::new(rule_set, false);

        let result = runner.run();

        assert!(result.is_err());
        assert!(runner.matched_files.is_empty());
    }
}
