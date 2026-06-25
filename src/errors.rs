use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FileError {
    #[error("path not found: {0}")]
    NotFound(PathBuf),

    #[error("invalid config file")]
    InvalidYaml(#[source] serde_yaml::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("expected file found directory: {0}")]
    NotFile(PathBuf),

    #[error("expected directory found file: {0}")]
    NotDirectory(PathBuf),

    #[error("path has no file name: {0}")]
    MissingFileName(PathBuf),

    #[error("file already exists: {0}")]
    FileAlreadyExist(PathBuf),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StepExecutionError {
    #[error(transparent)]
    File(#[from] FileError),

    #[error("step '{0}' is not implemented")]
    NotImplemented(&'static str),
}
