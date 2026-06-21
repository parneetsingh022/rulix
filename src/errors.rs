use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FileError {
    #[error("file not found: {0}")]
    NotFound(PathBuf),

    #[error("invalid config file")]
    InvalidYaml(#[source] serde_yaml::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StepExecutionError {
    #[error(transparent)]
    File(#[from] FileError),

    #[error("step '{0}' is not implemented")]
    NotImplemented(&'static str),
}
