use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

use crate::{checksum::file_checksum_matches, errors::FileError};

fn move_op(from: &Path, to: &Path, checksum: Option<&str>) -> Result<(), FileError> {
    if from.is_dir() {
        return Err(FileError::ExpectedFileFoundDirectory(from.to_path_buf()));
    }

    if to.exists() {
        return Err(FileError::TargetAlreadyExists(to.to_path_buf()));
    }

    // Verify that the file has not been modified since the operation was
    // recorded. Undo operations are only permitted when the file's current
    // contents match the original checksum.
    if let Some(c) = checksum
        && !file_checksum_matches(from, c)?
    {
        return Err(FileError::FileContentsChanged(from.to_path_buf()));
    }

    match fs::rename(from, to) {
        Ok(_) => Ok(()),
        Err(e) => {
            // EXDEV error: Invalid cross-device link
            if e.kind() == io::ErrorKind::CrossesDevices {
                fs::copy(from, to)?;
                fs::remove_file(from)?;
                return Ok(());
            }

            Err(FileError::Io(e))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Operation {
    Move {
        // Paths as provided by the caller (may be absolute or relative)
        from: Box<Path>,
        to: Box<Path>,
        checksum: Option<String>,
    },
}

impl Operation {
    pub fn execute(&self) -> Result<(), FileError> {
        match self {
            Operation::Move {
                from, to, checksum, ..
            } => move_op(from, to, checksum.as_deref())?,
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
            Operation::Move { from, to, checksum } => {
                Operation::Move {
                    // Swapping 'from' and 'to' cleanly reverses the action
                    from: to.clone(),
                    to: from.clone(),
                    checksum: checksum.clone(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn move_undo_operation_produces_right_results() {
        let move_op = Operation::Move {
            from: PathBuf::from("C:\\Users\\Parneet\\Desktop\\report.pdf").into_boxed_path(),
            to: PathBuf::from("C:\\Users\\Documents\\PDFs\\report.pdf").into_boxed_path(),
            checksum: None,
        };

        let expected_undo_op = Operation::Move {
            from: PathBuf::from("C:\\Users\\Documents\\PDFs\\report.pdf").into_boxed_path(),
            to: PathBuf::from("C:\\Users\\Parneet\\Desktop\\report.pdf").into_boxed_path(),
            checksum: None,
        };

        assert_eq!(move_op.get_undo_operation(), expected_undo_op);
    }

    #[test]
    fn move_undo_preserves_checksum_data() {
        let hash_string = Some(String::from(
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ));

        let move_with_hash = Operation::Move {
            from: PathBuf::from("source.txt").into_boxed_path(),
            to: PathBuf::from("dest.txt").into_boxed_path(),
            checksum: hash_string.clone(),
        };

        let expected_undo_op = Operation::Move {
            from: PathBuf::from("dest.txt").into_boxed_path(),
            to: PathBuf::from("source.txt").into_boxed_path(),
            checksum: hash_string,
        };

        assert_eq!(move_with_hash.get_undo_operation(), expected_undo_op);
    }

    #[test]
    fn double_undo_restores_original_operation() {
        let original_op = Operation::Move {
            from: PathBuf::from("relative/path/a.txt").into_boxed_path(),
            to: PathBuf::from("relative/path/b.txt").into_boxed_path(),
            checksum: Some(String::from("hash-123")),
        };

        // op.undo().undo() == op
        let double_inverted_op = original_op.get_undo_operation().get_undo_operation();

        assert_eq!(double_inverted_op, original_op);
    }

    #[test]
    fn move_undo_handles_empty_and_relative_paths() {
        let edge_case_op = Operation::Move {
            from: PathBuf::from(".").into_boxed_path(),
            to: PathBuf::from("").into_boxed_path(),
            checksum: None,
        };

        let expected_undo = Operation::Move {
            from: PathBuf::from("").into_boxed_path(),
            to: PathBuf::from(".").into_boxed_path(),
            checksum: None,
        };

        assert_eq!(edge_case_op.get_undo_operation(), expected_undo);
    }
}
