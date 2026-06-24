use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("undo failed: nothing to undo")]
    NothingToUndo,

    #[error("path not found: {0}")]
    NotFound(PathBuf),

    #[error("expected a file but found a directory: {0}")]
    ExpectedFileFoundDirectory(PathBuf),

    #[error("target path already exists: {0}")]
    TargetAlreadyExists(PathBuf),

    #[error("file contents changed: {0}")]
    FileContentsChanged(PathBuf),
}
