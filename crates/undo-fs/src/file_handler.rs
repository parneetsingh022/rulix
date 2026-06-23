use crate::{Operation, checksum::get_file_hash, errors::FileError};
use std::path::Path;

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
        let hash = get_file_hash(&from)?;

        let op = Operation::Move {
            from: from.as_ref().to_path_buf(),
            to: to.as_ref().to_path_buf(),
            is_dir: false,
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

        while let Some(op) = self.operations.pop() {
            op.undo()?;
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
    fn test_undo_fail_when_file_contents_changed() -> Result<(), FileError> {
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
}
