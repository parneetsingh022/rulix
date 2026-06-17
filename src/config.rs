use anyhow::Result;
use serde::Deserialize;
use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;

use crate::errors::ConfigError;

#[derive(Debug, Deserialize)]
pub struct Rule {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct RulixConfig {
    rules: Vec<Rule>,
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
