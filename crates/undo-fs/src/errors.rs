use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileError {
    #[error("IO operational error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse JSON line: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Undo failed, nothing to undo")]
    NothingToUndo,

    #[error("file not found {0}")]
    NotFound(PathBuf),

    #[error("file contents changed {0}")]
    FileContentsChanged(PathBuf),
}
