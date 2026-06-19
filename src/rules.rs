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
use std::{
    fs::File,
    io::ErrorKind,
    path::{Path, PathBuf},
};

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
/// - [`RulesFileSource::Default`] represents the default rules file location
///   chosen by Rulix. If the file does not exist, commands may treat this
///   as a valid state (for example, by reporting that no rules are
///   configured).
///
/// - [`RulesFileSource::User`] represents a path explicitly provided by the
///   user, such as via a command-line argument, i.e. `--rules <filename>`. If the file does not
///   exist, commands should generally treat this as an error and report
///   the missing path to the user.
pub enum RulesFileSource {
    Default(PathBuf),
    User(PathBuf),
}

impl RulesFileSource {
    pub fn path(&self) -> &Path {
        match self {
            RulesFileSource::Default(path) | RulesFileSource::User(path) => path,
        }
    }

    pub fn is_user_provided(&self) -> bool {
        matches!(self, RulesFileSource::User(_))
    }
}

/// A single rule definition loaded from a rules file.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub struct Rule {
    pub name: String,
    pub target: PathBuf,
    pub steps: Vec<Step>,
}

#[allow(dead_code)]
impl Rule {
    /// Returns steps in execution order.
    pub fn steps(&self) -> impl Iterator<Item = &Step> {
        self.steps.iter()
    }
}

/// Collection of all rules defined in a configuration file.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RuleSet {
    pub rules: Vec<Rule>,
}

impl RuleSet {
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
    fn default_rules_file_returns_rules_yaml_inside_system_config_dir() {
        let path = default_rules_file();

        assert_eq!(path.file_name().unwrap(), "rules.yaml");

        #[cfg(target_os = "windows")]
        assert_eq!(
            path,
            PathBuf::from("C:\\ProgramData")
                .join(".rulix")
                .join("rules.yaml")
        );

        #[cfg(not(target_os = "windows"))]
        assert_eq!(
            path,
            PathBuf::from("/etc")
                .join(".rulix")
                .join("rules.yaml")
        );
    }

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

        let config = RuleSet::from_file(&path).unwrap();

        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.rules[0].name, "organize-desktop");
        assert_eq!(
            config.rules[0].target.to_string_lossy(),
            "C:\\Users\\Parneet\\Desktop\\"
        );
        assert_eq!(
            config.rules[0].steps,
            vec![
                Step::new_match("pdf"),
                Step::new_move_to("C:\\Users\\Documents\\PDFs\\"),
                Step::new_notify("Large PDF moved successfully"),
            ]
        );

        assert_eq!(config.rules[1].name, "clean-downloads");
        assert_eq!(
            config.rules[1].target.to_string_lossy(),
            "C:\\Users\\Parneet\\Downloads\\"
        );
        assert_eq!(
            config.rules[1].steps,
            vec![
                Step::new_match("exe"),
                Step::new_move_to("C:\\Users\\Parneet\\Downloads\\Executables"),
                Step::new_notify("Moved exe files to Executables folder"),
            ]
        );
    }

    #[test]
    fn returns_not_found_for_missing_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.yaml");

        let err = RuleSet::from_file(&path).unwrap_err();

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

        let err = RuleSet::from_file(&path).unwrap_err();

        assert!(matches!(err, FileError::InvalidYaml(_)));
    }
}
