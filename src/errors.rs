use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    NotFound(String),

    #[error("invalid config file")]
    InvalidYaml(#[source] serde_yaml::Error),
}
