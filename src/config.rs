use anyhow::Result;
use serde::Deserialize;
use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;

use crate::errors::ConfigError;

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RulixConfig {
    pub rules: Vec<Rule>,
}

impl RulixConfig {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                return Err(ConfigError::NotFound(path.display().to_string()).into());
            }
            Err(e) => return Err(e.into()),
        };

        // Syntax error while parsing yaml file
        let config = serde_yaml::from_reader(file).map_err(ConfigError::InvalidYaml)?;

        Ok(config)
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

        let config = RulixConfig::from_file(&path).unwrap();

        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.rules[0].name, "Rule 1");
    }

    #[test]
    fn returns_not_found_for_missing_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.yaml");

        let err = RulixConfig::from_file(&path).unwrap_err();

        assert!(err.downcast_ref::<ConfigError>().is_some());
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

        let err = RulixConfig::from_file(&path).unwrap_err();

        let config_err = err.downcast_ref::<ConfigError>().unwrap();

        assert!(matches!(config_err, ConfigError::InvalidYaml(_)));
    }
}
