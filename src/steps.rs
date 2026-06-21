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

use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::errors::{FileError, StepExecutionError};

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
                handle_match(target, criteria, matched_files)?;
                Ok(())
            }

            Step::MoveTo { .. } => Err(StepExecutionError::NotImplemented("move_to")),

            Step::Notify { notify } => handle_notify(notify.as_str()),
        }
    }
}

fn handle_match(
    target: &Path,
    criteria: &MatchCriteria,
    matched_files: &mut Vec<PathBuf>,
) -> Result<(), FileError> {
    if !target.exists() {
        return Err(FileError::NotFound(target.to_path_buf()));
    }

    // Clear previously filtered files to support nested matching pipelines.
    // A single rule may chain multiple actions after a match step, followed by
    // subsequent match steps to filter new subsets of files. Clearing the
    // `matched_files` vector retains its allocated capacity, avoiding
    // reallocation overhead during these sequential operations.
    matched_files.clear();

    for entry in fs::read_dir(target)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.is_file() && criteria.matches(file_path.as_path()) {
            matched_files.push(file_path);
        }
    }

    Ok(())
}

fn handle_notify(arg: &str) -> Result<(), StepExecutionError> {
    println!("{}", arg);

    Ok(())
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
    use tempfile::tempdir;

    #[test]
    fn handle_match_returns_not_found_when_target_does_not_exist() {
        let temp_dir = tempdir().unwrap();

        let missing_path = temp_dir.path().join("does-not-exist");

        let criteria = MatchCriteria {
            ext: Some("txt".to_string()),
        };

        let mut matched_files = Vec::new();

        let err = handle_match(&missing_path, &criteria, &mut matched_files).unwrap_err();

        let error_message = err.to_string();

        assert!(error_message.contains("does-not-exist"));
        assert!(matched_files.is_empty());
    }

    #[test]
    fn match_returns_file_with_extension_containing_leading_dot() {
        let temp_dir = tempdir().unwrap();
        let text_file = temp_dir.path().join("file.txt");
        std::fs::write(&text_file, "hello").unwrap();

        let criteria = MatchCriteria {
            ext: Some(".txt".to_string()),
        };

        let mut matched_files: Vec<PathBuf> = Vec::new();

        handle_match(temp_dir.path(), &criteria, &mut matched_files).unwrap();

        assert_eq!(matched_files, vec![text_file]);
    }

    #[test]
    fn handle_match_adds_only_files_matching_extension() {
        let temp_dir = tempdir().unwrap();

        let txt_file = temp_dir.path().join("hello.txt");
        let another_txt_file = temp_dir.path().join("notes.txt");
        let rs_file = temp_dir.path().join("main.rs");
        let extensionless_file = temp_dir.path().join("README");
        let txt_dir = temp_dir.path().join("folder.txt");

        std::fs::write(&txt_file, "hello").unwrap();
        std::fs::write(&another_txt_file, "notes").unwrap();
        std::fs::write(&rs_file, "fn main() {}").unwrap();
        std::fs::write(&extensionless_file, "readme").unwrap();
        std::fs::create_dir(&txt_dir).unwrap();

        let nested_txt_file = txt_dir.join("nested.txt");
        std::fs::write(&nested_txt_file, "nested").unwrap();

        let criteria = MatchCriteria {
            ext: Some("txt".to_string()),
        };

        let mut matched_files = Vec::new();

        handle_match(temp_dir.path(), &criteria, &mut matched_files).unwrap();

        matched_files.sort();

        let mut expected = vec![txt_file, another_txt_file];
        expected.sort();

        assert_eq!(matched_files, expected);
    }

    #[test]
    fn notify_step_executes_successfully() {
        let step = Step::new_notify("hello world");
        let mut matched_files = Vec::new();

        assert!(step.execute(Path::new("."), &mut matched_files).is_ok());
    }
}
