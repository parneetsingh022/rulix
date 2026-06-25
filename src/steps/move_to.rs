use crate::errors::FileError;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn execute(target_dir: &Path, matched_files: &Vec<PathBuf>) -> Result<(), FileError> {
    for file in matched_files {
        let move_path = resolve_path(file, target_dir)?;

        move_file(file, &move_path)?;
    }

    Ok(())
}

fn move_file(source_file: &Path, target_file: &Path) -> Result<(), FileError> {
    ensure_file(source_file)?;
    ensure_file_absent(target_file)?;

    match fs::rename(source_file, target_file) {
        Ok(_) => Ok(()),
        // `rename` is usually an atomic move, but it only works
        // within same filesystem/device. If the source and destination
        // are on different devices fallback to copy + delete to
        // complete the move
        Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
            fs::copy(source_file, target_file)?;
            fs::remove_file(source_file)?;
            Ok(())
        }

        Err(e) => Err(FileError::Io(e)),
    }
}

/// Builds the destination path for moving `file_path` into `folder_path`.
///
/// The destination is formed by taking the final file name from `file_path` and
/// joining it onto `folder_path`.
/// Returns an error if `file_path` is not a file, if `folder_path` does not
/// exist, if `folder_path` is not a directory, or if `file_path` has no final
/// file name.
fn resolve_path(file_path: &Path, folder_path: &Path) -> Result<PathBuf, FileError> {
    ensure_file(file_path)?;
    ensure_dir(folder_path)?;

    let filename = file_path
        .file_name()
        .ok_or_else(|| FileError::NotFile(file_path.to_path_buf()))?;

    Ok(folder_path.join(filename))
}

fn ensure_file(file_path: &Path) -> Result<(), FileError> {
    if !file_path.exists() {
        return Err(FileError::NotFound(file_path.to_path_buf()));
    } else if !file_path.is_file() {
        return Err(FileError::NotFile(file_path.to_path_buf()));
    }

    Ok(())
}

fn ensure_file_absent(file: &Path) -> Result<(), FileError> {
    if file.exists() {
        return Err(FileError::FileAlreadyExist(file.to_path_buf()));
    }

    Ok(())
}

fn ensure_dir(folder_path: &Path) -> Result<(), FileError> {
    if !folder_path.exists() {
        return Err(FileError::NotFound(folder_path.to_path_buf()));
    } else if !folder_path.is_dir() {
        return Err(FileError::NotDirectory(folder_path.to_path_buf()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_path_with_right_file_paths() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let target_file = dir.path().join("test.txt");
        fs::write(&target_file, "Contents")?;

        let target_dir = dir.path().join("test_dir");
        fs::create_dir(&target_dir)?;

        let result_path = target_dir.join("test.txt");

        let result = resolve_path(&target_file, &target_dir)?;

        assert_eq!(result.as_path(), result_path);

        Ok(())
    }

    #[test]
    fn test_resolve_raises_error_when_file_path_is_dir() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let target_file = dir.path().join("folder");
        fs::create_dir(&target_file)?;

        let target_dir = dir.path().join("test_dir");
        fs::create_dir(&target_dir)?;

        let result = resolve_path(&target_file, &target_dir);

        assert!(
            matches!(&result, Err(FileError::NotFile(_))),
            "Assertion Failed! Expected Err(FileError::IsDirectory), got {result:?}"
        );

        Ok(())
    }

    #[test]
    fn test_resolve_raises_error_when_target_is_existing_file() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let file = dir.path().join("file1.txt");
        fs::write(&file, "content")?;

        let target_dir = dir.path().join("test_dir");
        fs::create_dir(&target_dir)?;

        let target_file = target_dir.join("test.txt");
        fs::write(&target_file, "content")?;

        let result = resolve_path(&file, &target_file);

        assert!(
            matches!(&result, Err(FileError::NotDirectory(_))),
            "Assertion Failed! Expected Err(FileError::NotDirectory), got {result:?}"
        );

        Ok(())
    }

    #[test]
    fn test_resolve_path_raises_error_when_target_dir_does_not_exist() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let target_file = dir.path().join("test.txt");
        fs::write(&target_file, "Contents")?;

        let target_dir = dir.path().join("test_dir");

        let result = resolve_path(&target_file, &target_dir);

        assert!(
            matches!(&result, Err(FileError::NotFound(_))),
            "Assertion Failed! Expected Err(FileError::NotFound), got {result:?}"
        );

        Ok(())
    }

    #[test]
    fn test_resolve_path_raises_error_when_source_does_not_exist() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let target_file = dir.path().join("test.txt");

        let target_dir = dir.path().join("test_dir");

        let result = resolve_path(&target_file, &target_dir);

        assert!(
            matches!(&result, Err(FileError::NotFound(_))),
            "Assertion Failed! Expected Err(FileError::NotFound), got {result:?}"
        );

        Ok(())
    }
}
