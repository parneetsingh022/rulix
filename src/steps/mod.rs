//! Workflow configuration types.
//!
//! Defines the structures used to deserialize the `steps` section of a Rulix
//! rule file. Each YAML step maps to a [`Step`] variant, which represents a
//! single action in the rule execution pipeline.
//!
//! Steps are executed sequentially in the order they appear in the
//! configuration file. Each step represents a discrete operation within
//! the workflow pipeline.
//!
//! This module serves as the configuration layer between YAML rule files and
//! the runtime execution engine.

mod match_files;
mod notify;

use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::errors::StepExecutionError;

/// Rule filters used to evaluate whether a file matches a given `Step::Match`.
#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct MatchCriteria {
    /// The target file extension (e.g., `"pdf"`, `"docx"`).
    pub ext: Option<String>,
}

impl MatchCriteria {
    pub fn matches(&self, path: &Path) -> bool {
        if let Some(ext) = &self.ext {
            // Allow user to add extinsion with or without a leading `.`
            let expected_ext = ext.trim_start_matches(".");
            let file_ext = path.extension().and_then(|e| e.to_str());

            if file_ext != Some(expected_ext) {
                return false;
            }
        }

        true
    }
}

/// Defines a single operation within a rule's workflow pipeline.
///
/// Each step is deserialized from YAML and executed sequentially by
/// the engine as part of rule processing.
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Step {
    /// Evaluates whether the current file satisfies the given criteria.
    Match {
        #[serde(rename = "match")]
        criteria: MatchCriteria,
    },

    /// Relocates the targeted file to a specified destination directory.
    MoveTo { move_to: PathBuf },

    /// Displays a message on terminal.
    Notify { notify: String },
}

impl Step {
    pub fn execute(
        &self,
        target: &Path,
        matched_files: &mut Vec<PathBuf>,
    ) -> Result<(), StepExecutionError> {
        match self {
            // Match: fetches files from the target directory matching the criteria, then populates the matched_files vector.
            Step::Match { criteria } => {
                match_files::execute(target, criteria, matched_files)?;
                Ok(())
            }

            Step::MoveTo { .. } => Err(StepExecutionError::NotImplemented("move_to")),

            Step::Notify { notify } => notify::execute(notify.as_str()),
        }
    }
}

/// Convenience constructors used by unit tests.
#[cfg(test)]
impl Step {
    pub fn new_match(ext: &str) -> Self {
        Self::Match {
            criteria: MatchCriteria {
                ext: Some(ext.to_string()),
            },
        }
    }

    pub fn new_move_to(path: &str) -> Self {
        Self::MoveTo {
            move_to: PathBuf::from(path),
        }
    }

    pub fn new_notify(msg: &str) -> Self {
        Self::Notify {
            notify: msg.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notify_step_executes_successfully() {
        let step = Step::new_notify("hello world");
        let mut matched_files = Vec::new();

        assert!(step.execute(Path::new("."), &mut matched_files).is_ok());
    }
}
