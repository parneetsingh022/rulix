use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{checksum::file_checksum_matches, errors::FileError};

fn move_op(from: &Path, to: &Path, checksum: Option<&str>) -> Result<(), FileError> {
    // Verify that the file has not been modified since the operation was
    // recorded. Undo operations are only permitted when the file's current
    // contents match the original checksum.
    if let Some(c) = checksum
        && !file_checksum_matches(from, c)?
    {
        return Err(FileError::FileContentsChanged(from.to_path_buf()));
    }

    fs::rename(from, to)?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Operation {
    Move {
        // Use full, complete absolute paths for clear tracking
        from: PathBuf,
        to: PathBuf,
        is_dir: bool,
        checksum: Option<String>,
    },
}

impl Operation {
    pub fn execute(&self) -> Result<(), FileError> {
        match self {
            Operation::Move {
                from, to, checksum, ..
            } => move_op(from.as_path(), to.as_path(), checksum.as_deref())?,
        }

        Ok(())
    }

    pub fn undo(&self) -> Result<(), FileError> {
        self.get_undo_operation().execute()?;

        Ok(())
    }

    /// Generates a perfectly inverted operation without any risk of panics.
    pub fn get_undo_operation(&self) -> Self {
        match self {
            Operation::Move {
                from,
                to,
                is_dir,
                checksum,
            } => {
                Operation::Move {
                    // Swapping 'from' and 'to' cleanly reverses the action
                    from: to.clone(),
                    to: from.clone(),
                    is_dir: *is_dir,
                    checksum: checksum.clone(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_undo_operation_produces_right_results() {
        let move_op = Operation::Move {
            from: PathBuf::from("C:\\Users\\Parneet\\Desktop\\report.pdf"),
            to: PathBuf::from("C:\\Users\\Documents\\PDFs\\report.pdf"),
            is_dir: false,
            checksum: None,
        };

        let expected_undo_op = Operation::Move {
            from: PathBuf::from("C:\\Users\\Documents\\PDFs\\report.pdf"),
            to: PathBuf::from("C:\\Users\\Parneet\\Desktop\\report.pdf"),
            is_dir: false,
            checksum: None,
        };

        assert_eq!(move_op.get_undo_operation(), expected_undo_op);
    }

    #[test]
    fn move_undo_preserves_directory_flag() {
        let move_dir_op = Operation::Move {
            from: PathBuf::from("/etc/source_dir"),
            to: PathBuf::from("/etc/target_dir"),
            is_dir: true, // Testing directory variant
            checksum: None,
        };

        let expected_undo_op = Operation::Move {
            from: PathBuf::from("/etc/target_dir"),
            to: PathBuf::from("/etc/source_dir"),
            is_dir: true,
            checksum: None,
        };

        assert_eq!(move_dir_op.get_undo_operation(), expected_undo_op);
    }

    #[test]
    fn move_undo_preserves_checksum_data() {
        let hash_string = Some(String::from(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ));

        let move_with_hash = Operation::Move {
            from: PathBuf::from("source.txt"),
            to: PathBuf::from("dest.txt"),
            is_dir: false,
            checksum: hash_string.clone(),
        };

        let expected_undo_op = Operation::Move {
            from: PathBuf::from("dest.txt"),
            to: PathBuf::from("source.txt"),
            is_dir: false,
            checksum: hash_string,
        };

        assert_eq!(move_with_hash.get_undo_operation(), expected_undo_op);
    }

    #[test]
    fn double_undo_restores_original_operation() {
        let original_op = Operation::Move {
            from: PathBuf::from("relative/path/a.txt"),
            to: PathBuf::from("relative/path/b.txt"),
            is_dir: false,
            checksum: Some(String::from("hash-123")),
        };

        // op.undo().undo() == op
        let double_inverted_op = original_op.get_undo_operation().get_undo_operation();

        assert_eq!(double_inverted_op, original_op);
    }

    #[test]
    fn move_undo_handles_empty_and_relative_paths() {
        let edge_case_op = Operation::Move {
            from: PathBuf::from("."),
            to: PathBuf::from(""),
            is_dir: false,
            checksum: None,
        };

        let expected_undo = Operation::Move {
            from: PathBuf::from(""),
            to: PathBuf::from("."),
            is_dir: false,
            checksum: None,
        };

        assert_eq!(edge_case_op.get_undo_operation(), expected_undo);
    }
}
