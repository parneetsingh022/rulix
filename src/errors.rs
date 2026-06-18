use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FileError {
    #[error("config file not found: {0}")]
    NotFound(String),

    #[error("invalid config file")]
    InvalidYaml(#[source] serde_yaml::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
