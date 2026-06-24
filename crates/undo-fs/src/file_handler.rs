use crate::{Operation, checksum::get_file_hash, errors::FileError};
use std::path::{Path, PathBuf};

/// Resolves the final destination path for a move operation.
///
/// If `to` refers to an existing directory, the file name from `from` is
/// appended to it and the resulting path is returned. Otherwise, `to` is
/// treated as the complete destination path and returned unchanged.
///
/// This mirrors the behavior of common file move utilities, where moving a
/// file to a directory preserves its file name, while moving it to a file
/// path effectively renames it.
fn resolve_move_destination(from: &Path, to: &Path) -> PathBuf {
    if !to.is_dir() {
        return to.to_path_buf();
    }

    let Some(filename) = from.file_name() else {
        return to.to_path_buf();
    };

    to.join(filename)
}

#[derive(Default)]
pub struct FileHandler {
    operations: Vec<Operation>,
}

impl FileHandler {
    pub fn new() -> Self {
        FileHandler::default()
    }

    pub fn move_file(
        &mut self,
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) -> Result<(), FileError> {
        let from = from.as_ref();
        let to = to.as_ref();

        let hash = get_file_hash(from)?;

        // Resolve the final move destination. If `to` is an existing directory,
        // preserve the source file name by moving into that directory; otherwise
        // treat `to` as the complete destination path (i.e. a rename target).
        let target = resolve_move_destination(from, to);

        let op = Operation::Move {
            from: from.into(),
            to: target.into_boxed_path(),
            checksum: Some(hash),
        };

        // Only record the operation in our history if it executes successfully.
        // This prevents failed operations from polluting the undo stack.
        op.execute()?;

        self.operations.push(op);

        Ok(())
    }

    pub fn undo(&mut self) -> Result<(), FileError> {
        let Some(op) = self.operations.pop() else {
            return Err(FileError::NothingToUndo);
        };

        if let Err(e) = op.undo() {
            self.operations.push(op);
            return Err(e);
        }

        Ok(())
    }

    pub fn undo_all(&mut self) -> Result<(), FileError> {
        if self.operations.is_empty() {
            return Err(FileError::NothingToUndo);
        };

        while !self.operations.is_empty() {
            self.undo()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use tempfile::TempDir;

    #[test]
    fn move_file_and_undo_with_file_handler() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let file1 = dir.path().join("file.txt");
        fs::write(&file1, "File Contents")?;

        let file2 = dir.path().join("file2.txt");

        assert!(file1.is_file());

        let mut fh = FileHandler::new();
        fh.move_file(&file1, &file2)?;

        assert!(!file1.is_file());
        assert!(file2.is_file());
        assert_eq!(fs::read(&file2)?, b"File Contents");

        fh.undo()?;
        assert!(file1.is_file());
        assert!(!file2.is_file());
        assert_eq!(fs::read(&file1)?, b"File Contents");

        Ok(())
    }

    #[test]
    fn move_file_fails_when_source_does_not_exist() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let file1 = dir.path().join("missing.txt");
        let file2 = dir.path().join("file2.txt");

        let mut fh = FileHandler::new();
        let result = fh.move_file(&file1, &file2);

        assert!(matches!(
            result,
            Err(FileError::NotFound(path)) if path == file1
        ));

        assert!(!file1.exists());
        assert!(!file2.exists());

        Ok(())
    }

    #[test]
    fn move_file_fails_when_file_already_exists() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let file1 = dir.path().join("file.txt");
        fs::write(&file1, "File 1 Contents")?;

        let file2 = dir.path().join("file2.txt");
        fs::write(&file2, "File 2 Contents")?;

        assert!(file1.is_file());
        assert!(file2.is_file());

        let mut fh = FileHandler::new();
        let result = fh.move_file(&file1, &file2);

        assert!(matches!(
            result.err(),
            Some(FileError::TargetAlreadyExists(path)) if path == file2
        ));

        assert!(file1.is_file());
        assert_eq!(fs::read(&file1)?, b"File 1 Contents");

        assert!(file2.is_file());
        assert_eq!(fs::read(&file2)?, b"File 2 Contents");

        Ok(())
    }

    #[test]
    fn move_file_to_directory_fails_if_file_with_same_name_exists() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let file1 = dir.path().join("file.txt");
        fs::write(&file1, "Original contents")?;

        let target_dir = dir.path().join("target_dir");
        fs::create_dir(&target_dir)?;

        let existing_target = target_dir.join("file.txt");
        fs::write(&existing_target, "Existing contents")?;

        let mut fh = FileHandler::new();
        let result = fh.move_file(&file1, &target_dir);

        match result {
            Err(FileError::TargetAlreadyExists(path)) => {
                assert_eq!(path, existing_target);
            }
            other => panic!("Expected TargetAlreadyExists, got {:?}", other),
        }

        assert!(file1.exists());
        assert!(existing_target.exists());
        assert_eq!(fs::read(&file1)?, b"Original contents");
        assert_eq!(fs::read(&existing_target)?, b"Existing contents");

        Ok(())
    }

    #[test]
    fn move_file_to_a_folder_path() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let file1 = dir.path().join("file.txt");
        fs::write(&file1, "File 1 Contents")?;

        let new_dir = dir.path().join("new_dir");
        fs::create_dir(&new_dir)?;

        let new_file = new_dir.join("file.txt");

        assert!(file1.is_file());
        assert!(new_dir.is_dir());
        assert!(!new_file.is_file());

        let mut fh = FileHandler::new();
        fh.move_file(&file1, &new_dir)?;

        assert!(!file1.is_file());
        assert!(new_file.is_file());
        assert_eq!(fs::read(&new_file)?, b"File 1 Contents");

        fh.undo()?;

        assert!(file1.is_file());
        assert!(!new_file.is_file());
        assert_eq!(fs::read(&file1)?, b"File 1 Contents");

        Ok(())
    }

    #[test]
    fn complex_multi_move_and_undo_all() -> Result<(), FileError> {
        let base_dir = TempDir::new()?;

        let dir_a = base_dir.path().join("folder_a");
        let dir_b = base_dir.path().join("folder_b");
        let dir_c = base_dir.path().join("folder_c");

        fs::create_dir(&dir_a)?;
        fs::create_dir(&dir_b)?;
        fs::create_dir(&dir_c)?;

        // File names
        let f1_name = "doc1.txt";
        let f2_name = "doc2.txt";

        // Initial paths in folder_a
        let a_file1 = dir_a.join(f1_name);
        let a_file2 = dir_a.join(f2_name);

        // Target paths in folder_b
        let b_file1 = dir_b.join(f1_name);
        let b_file2 = dir_b.join(f2_name);

        // Target paths in folder_c
        let c_file1 = dir_c.join(f1_name);
        let c_file2 = dir_c.join(f2_name);

        // Write initial data
        fs::write(&a_file1, "Content of Document 1")?;
        fs::write(&a_file2, "Content of Document 2")?;

        let mut fh = FileHandler::new();

        // Move both files from A to B
        fh.move_file(&a_file1, &b_file1)?;
        fh.move_file(&a_file2, &b_file2)?;

        assert!(!a_file1.exists() && !a_file2.exists());
        assert!(b_file1.is_file() && b_file2.is_file());

        // Move file1 from B to C, and file2 back from B to A (criss-cross)
        fh.move_file(&b_file1, &c_file1)?;
        fh.move_file(&b_file2, &a_file2)?;

        // Move file2 from A to C as well
        fh.move_file(&a_file2, &c_file2)?;

        // Final layout verification before rolling back: Both files should be in C
        assert!(!a_file1.exists() && !a_file2.exists());
        assert!(!b_file1.exists() && !b_file2.exists());
        assert!(c_file1.is_file() && c_file2.is_file());
        assert_eq!(fs::read(&c_file1)?, b"Content of Document 1");
        assert_eq!(fs::read(&c_file2)?, b"Content of Document 2");

        fh.undo_all()?;

        // Folder C must be completely empty
        assert!(!c_file1.exists());
        assert!(!c_file2.exists());

        // Folder B must be completely empty
        assert!(!b_file1.exists());
        assert!(!b_file2.exists());

        // Folder A must contain the original files with their precise content
        assert!(a_file1.is_file());
        assert!(a_file2.is_file());
        assert_eq!(fs::read(&a_file1)?, b"Content of Document 1");
        assert_eq!(fs::read(&a_file2)?, b"Content of Document 2");

        Ok(())
    }

    #[test]
    fn move_fails_when_source_is_directory() -> Result<(), FileError> {
        let base_dir = TempDir::new()?;

        let folder1 = base_dir.path().join("folder1");
        fs::create_dir(&folder1)?;

        let folder2 = base_dir.path().join("folder2");

        let mut fh = FileHandler::new();
        let result = fh.move_file(&folder1, &folder2);

        match result {
            Err(FileError::ExpectedFileFoundDirectory(path)) => {
                assert_eq!(path, folder1);
            }

            other => panic!(
                "Expected error: ExpectedFileFoundDirectory, got {:?}",
                other
            ),
        }

        Ok(())
    }

    #[test]
    fn undo_fail_when_file_contents_changed() -> Result<(), FileError> {
        let base_dir = TempDir::new()?;

        let file1 = base_dir.path().join("file.txt");
        let file2 = base_dir.path().join("file2.txt");

        fs::write(&file1, "Content")?;

        let mut fh = FileHandler::new();
        fh.move_file(&file1, &file2)?;

        fs::write(&file2, "New Content")?;

        match fh.undo() {
            Err(FileError::FileContentsChanged(path)) => {
                assert_eq!(path, file2);
            }
            other => panic!("Expected FileContentsChanged, got {:?}", other),
        }

        assert!(file2.exists());
        assert!(!file1.exists());

        Ok(())
    }

    #[test]
    fn undo_failure_keeps_operation_so_it_can_be_retried() -> Result<(), FileError> {
        let base_dir = TempDir::new()?;

        let file1 = base_dir.path().join("file.txt");
        let file2 = base_dir.path().join("file2.txt");

        fs::write(&file1, "Content")?;

        let mut fh = FileHandler::new();
        fh.move_file(&file1, &file2)?;

        fs::write(&file2, "New Content")?;

        assert!(matches!(
            fh.undo(),
            Err(FileError::FileContentsChanged(path)) if path == file2
        ));

        assert!(file2.exists());
        assert!(!file1.exists());

        // Resolve the issue
        fs::write(&file2, "Content")?;

        // Retry should now succeed because the failed operation was pushed back
        fh.undo()?;

        assert!(file1.exists());
        assert!(!file2.exists());
        assert_eq!(fs::read(&file1)?, b"Content");

        Ok(())
    }

    #[test]
    fn undo_all_failure_keeps_failing_operation_so_it_can_be_retried() -> Result<(), FileError> {
        let base_dir = TempDir::new()?;

        let file1 = base_dir.path().join("file.txt");
        let file2 = base_dir.path().join("file2.txt");
        let file3 = base_dir.path().join("file3.txt");

        fs::write(&file1, "Content")?;

        let mut fh = FileHandler::new();
        fh.move_file(&file1, &file2)?;
        fh.move_file(&file2, &file3)?;

        fs::write(&file3, "New Content")?;

        assert!(matches!(
            fh.undo_all(),
            Err(FileError::FileContentsChanged(path)) if path == file3
        ));

        assert!(file3.exists());
        assert!(!file2.exists());
        assert!(!file1.exists());

        // Resolve the issue
        fs::write(&file3, "Content")?;

        // Retry should now undo both operations
        fh.undo_all()?;

        assert!(file1.exists());
        assert!(!file2.exists());
        assert!(!file3.exists());
        assert_eq!(fs::read(&file1)?, b"Content");

        Ok(())
    }

    #[test]
    #[ignore = "requires MOVE_TEST_SRC and MOVE_TEST_DST on different filesystems"]
    fn cross_device_move_file_and_undo() -> Result<(), FileError> {
        let Ok(src_dir) = std::env::var("MOVE_TEST_SRC") else {
            eprintln!("Skipping cross-device test: MOVE_TEST_SRC not set");
            return Ok(());
        };

        let Ok(dst_dir) = std::env::var("MOVE_TEST_DST") else {
            eprintln!("Skipping cross-device test: MOVE_TEST_DST not set");
            return Ok(());
        };

        let src_dir = PathBuf::from(src_dir);
        let dst_dir = PathBuf::from(dst_dir);

        fs::create_dir_all(&src_dir)?;
        fs::create_dir_all(&dst_dir)?;

        let file1 = src_dir.join("cross-device-file.txt");
        let file2 = dst_dir.join("cross-device-file.txt");

        let _ = fs::remove_file(&file1);
        let _ = fs::remove_file(&file2);

        fs::write(&file1, "Cross device contents")?;

        let mut fh = FileHandler::new();
        fh.move_file(&file1, &file2)?;

        assert!(!file1.exists());
        assert!(file2.is_file());
        assert_eq!(fs::read(&file2)?, b"Cross device contents");

        fh.undo()?;

        assert!(file1.is_file());
        assert!(!file2.exists());
        assert_eq!(fs::read(&file1)?, b"Cross device contents");

        Ok(())
    }
}
