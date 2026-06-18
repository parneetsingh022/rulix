use serde::Deserialize;
use std::path::PathBuf;
use std::{fs::File, io::ErrorKind, path::Path};

use crate::config::SYSTEM_CONFIG_DIR;
use crate::errors::FileError;

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
///   user, such as via a command-line argument. If the file does not
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

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub name: String,
}

#[derive(Debug, Deserialize)]
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

        Ok(serde_yaml::from_reader(file).map_err(FileError::InvalidYaml)?)
    }

    /// Returns total number of rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
                - name: "Rule 1"
                - name: "Rule 2"
            "#},
        )
        .unwrap();

        let config = RulixRules::from_file(&path).unwrap();

        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.rules[0].name, "Rule 1");
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
