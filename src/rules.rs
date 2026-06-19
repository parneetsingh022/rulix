//! Rule file loading and configuration types.
//!
//! Defines the data structures used to deserialize `rules.yaml` and provides
//! helpers for loading rule files from disk.
//!
//! Each rule describes a target directory and an ordered workflow of
//! [`Step`]s that are executed when matching files are processed by the
//! Rulix engine.
//!
//! This module acts as the bridge between YAML rule files and the runtime
//! rule execution system.

use serde::Deserialize;
use std::path::PathBuf;
use std::{collections::VecDeque, fs::File, io::ErrorKind, path::Path};

use crate::config::SYSTEM_CONFIG_DIR;
use crate::errors::FileError;
use crate::steps::Step;

/// Returns the path to the default rules configuration file.
///
/// The file is expected to be located in the system configuration
/// directory and named `rules.yaml`.
///
/// # Returns
///
/// A [`PathBuf`] pointing to `<SYSTEM_CONFIG_DIR>/rules.yaml`.
pub fn default_rules_file() -> PathBuf {
    SYSTEM_CONFIG_DIR.join("rules.yaml")
}

/// Describes the origin of the rules file path.
///
/// This distinction is used to determine how missing files should be
/// handled.
///
/// - [`RulesSource::Default`] represents the default rules file location
///   chosen by Rulix. If the file does not exist, commands may treat this
///   as a valid state (for example, by reporting that no rules are
///   configured).
///
/// - [`RulesSource::User`] represents a path explicitly provided by the
///   user, such as via a command-line argument, i.e. `--rules <filename>`. If the file does not
///   exist, commands should generally treat this as an error and report
///   the missing path to the user.
pub enum RulesSource {
    Default(PathBuf),
    User(PathBuf),
}

impl RulesSource {
    pub fn path(&self) -> &Path {
        match self {
            RulesSource::Default(path) | RulesSource::User(path) => path,
        }
    }

    pub fn is_user_provided(&self) -> bool {
        matches!(self, RulesSource::User(_))
    }
}


/// A single rule definition loaded from a rules file.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Rule {
    pub name: String,
    pub target: PathBuf,
    pub steps: VecDeque<Step>,
}

impl Rule {
    // Returns an iterator over references to the steps
    pub fn pop_next_step(&mut self) -> Option<Step> {
        self.steps.pop_front()
    }
}


/// Collection of all rules defined in a configuration file.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RulixRules {
    pub rules: Vec<Rule>,
}

impl RulixRules {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, FileError> {
        let path = path.as_ref();

        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                return Err(FileError::NotFound(path.display().to_string()));
            }
            Err(e) => return Err(e.into()),
        };

        serde_yaml::from_reader(file).map_err(FileError::InvalidYaml)
    }

    /// Returns total number of rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steps::Step;
    use indoc::indoc;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn loads_valid_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.yaml");

        fs::write(
            &path,
            indoc! {r#"
            rules:
              - name: "organize-desktop"
                target: "C:\\Users\\Parneet\\Desktop\\"
                steps:
                  - match:
                      ext: "pdf" # extension
                  - move_to: "C:\\Users\\Documents\\PDFs\\"
                  - notify: "Large PDF moved successfully"
            
              - name: "clean-downloads"
                target: "C:\\Users\\Parneet\\Downloads\\"
                steps:
                  - match:
                      ext: "exe"
                  - move_to: "C:\\Users\\Parneet\\Downloads\\Executables"
                  - notify: "Moved exe files to Executables folder"

            "#},
        )
        .unwrap();

        let mut config = RulixRules::from_file(&path).unwrap();

        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.rules[0].name, "organize-desktop");
        assert_eq!(config.rules[0].target.to_string_lossy(), "C:\\Users\\Parneet\\Desktop\\");
        assert_eq!(
            config.rules[0].pop_next_step(),
            Some(Step::new_match("pdf"))
        );
        assert_eq!(
            config.rules[0].pop_next_step(),
            Some(Step::new_move_to("C:\\Users\\Documents\\PDFs\\"))
        );
        assert_eq!(
            config.rules[0].pop_next_step(),
            Some(Step::new_notify("Large PDF moved successfully"))
        );

        assert_eq!(config.rules[1].name, "clean-downloads");
        assert_eq!(config.rules[1].target.to_string_lossy(), "C:\\Users\\Parneet\\Downloads\\");
        assert_eq!(
            config.rules[1].pop_next_step(),
            Some(Step::new_match("exe"))
        );
        assert_eq!(
            config.rules[1].pop_next_step(),
            Some(Step::new_move_to(
                "C:\\Users\\Parneet\\Downloads\\Executables"
            ))
        );
        assert_eq!(
            config.rules[1].pop_next_step(),
            Some(Step::new_notify("Moved exe files to Executables folder"))
        );
    }

    #[test]
    fn returns_not_found_for_missing_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.yaml");

        let err = RulixRules::from_file(&path).unwrap_err();

        assert!(matches!(err, FileError::NotFound(_)));
    }

    #[test]
    fn returns_invalid_yaml_for_bad_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.yaml");

        fs::write(
            &path,
            indoc! {r#"
                rules:
                  -name: "Rule 1"
            "#},
        )
        .unwrap();

        let err = RulixRules::from_file(&path).unwrap_err();

        assert!(matches!(err, FileError::InvalidYaml(_)));
    }
}
