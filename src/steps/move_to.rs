use crate::errors::{FileError, StepExecutionError};
use std::{
    borrow::Cow,
    fs, io,
    path::{Path, PathBuf},
};

use console::style;

/// Moves all `matched_files` into `target_dir`
///
/// Each path in `matched_files` must point to a file not directory.
/// `target_dir` must be an existing directory.
pub fn execute(target_dir: &Path, matched_files: &Vec<PathBuf>) -> Result<(), FileError> {
    for file in matched_files {
        let move_path = resolve_path(file, target_dir)?;

        move_file(file, &move_path)?;
    }

    Ok(())
}

pub fn dry_run(
    target_dir: &Path,
    matched_files: &mut Vec<PathBuf>,
) -> Result<(), StepExecutionError> {
    if matched_files.is_empty() {
        println!(
            "{} {}",
            style("info").blue().bold(),
            style("No matching files found. Skipping step.").dim()
        );
        println!();
        return Ok(());
    }

    matched_files.sort();

    for file in matched_files {
        let move_path = resolve_path(file, target_dir)?;

        println!(
            "{} from {} to {}",
            style("MOVE").green(),
            style(format_path(file)).dim(),
            style(format_path(&move_path)).dim()
        );
    }

    println!();

    Ok(())
}

/// Moves `source_file` to `target_file`
///
/// The source path must exist and must be a file. The target path
/// must not already exist.
///
/// This function uses `fs::rename` to move/rename file. If the
/// source and target are on different filesystem/device it first
/// copies `source_file` to `target_file` then remove `source_file`.
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

fn format_path(path: &Path) -> Cow<'_, str> {
    let path = path.to_string_lossy();

    if path.contains("\\") {
        return Cow::Owned(path.replace("\\", "/"));
    }

    path
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
    fn format_path_returns_path_unchanged_when_it_has_forward_slashes() {
        let path = Path::new("dir1/file1.txt");

        let formatted = format_path(path);

        assert_eq!(formatted, "dir1/file1.txt");
    }

    #[test]
    fn format_path_replaces_backslashes_with_forward_slashes() {
        let path = Path::new(r"dir1\file1.txt");

        let formatted = format_path(path);

        assert_eq!(formatted, "dir1/file1.txt");
    }

    #[test]
    fn format_path_replaces_multiple_backslashes() {
        let path = Path::new(r"dir1\nested\file1.txt");

        let formatted = format_path(path);

        assert_eq!(formatted, "dir1/nested/file1.txt");
    }

    #[test]
    fn format_path_returns_borrowed_when_no_backslashes_exist() {
        let path = Path::new("dir1/file1.txt");

        let formatted = format_path(path);

        assert!(matches!(formatted, Cow::Borrowed(_)));
    }

    #[test]
    fn format_path_returns_owned_when_backslashes_exist() {
        let path = Path::new(r"dir1\file1.txt");

        let formatted = format_path(path);

        assert!(matches!(formatted, Cow::Owned(_)));
    }

    #[test]
    fn resolve_path_with_right_file_paths() -> Result<(), FileError> {
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
    fn resolve_raises_error_when_file_path_is_dir() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let source_file = dir.path().join("folder");
        fs::create_dir(&source_file)?;

        let target_dir = dir.path().join("test_dir");
        fs::create_dir(&target_dir)?;

        let result = resolve_path(&source_file, &target_dir);

        let path = match result {
            Err(FileError::NotFile(path)) => path,
            _ => panic!("Expected FileError::NotFound got {result:?}"),
        };

        assert_eq!(path, source_file);

        Ok(())
    }

    #[test]
    fn resolve_raises_error_when_target_is_existing_file() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let file = dir.path().join("file1.txt");
        fs::write(&file, "content")?;

        let target_dir = dir.path().join("test_dir");
        fs::create_dir(&target_dir)?;

        let target_file = target_dir.join("test.txt");
        fs::write(&target_file, "content")?;

        let result = resolve_path(&file, &target_file);

        let path = match result {
            Err(FileError::NotDirectory(path)) => path,
            _ => panic!("Expected Err(FileError::NotDirectory), got {result:?}"),
        };

        assert_eq!(path, target_file);

        Ok(())
    }

    #[test]
    fn resolve_path_raises_error_when_target_dir_does_not_exist() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let source_file = dir.path().join("test.txt");
        fs::write(&source_file, "Contents")?;

        let target_dir = dir.path().join("test_dir");

        let result = resolve_path(&source_file, &target_dir);

        let path = match result {
            Err(FileError::NotFound(path)) => path,
            _ => panic!("Expected Err(FileError::NotFound, got {result:?})"),
        };

        assert_eq!(path, target_dir);

        Ok(())
    }

    #[test]
    fn resolve_path_raises_error_when_source_does_not_exist() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let source_file = dir.path().join("test.txt");

        let target_dir = dir.path().join("test_dir");

        let result = resolve_path(&source_file, &target_dir);

        let error_path = match result {
            Err(FileError::NotFound(path)) => path,
            _ => panic!("Expected Err(FileError::NotFound), got {result:?}"),
        };

        assert_eq!(error_path, source_file);

        Ok(())
    }

    #[test]
    fn move_file_moves_from_source_to_target() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let source = dir.path().join("file1.txt");
        fs::write(&source, "Source File")?;

        let dest_dir = dir.path().join("dest");
        fs::create_dir(&dest_dir)?;

        let target = dest_dir.join("target.txt");

        move_file(&source, &target)?;
        assert!(!source.exists());
        assert!(target.exists());
        assert_eq!(fs::read(&target)?, "Source File".as_bytes());

        Ok(())
    }

    #[test]
    fn move_file_raises_error_when_source_does_not_exist() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let source = dir.path().join("file1.txt");

        let dest_dir = dir.path().join("dest");
        fs::create_dir(&dest_dir)?;

        let target = dest_dir.join("target.txt");

        let path = match move_file(&source, &target).unwrap_err() {
            FileError::NotFound(path) => path,
            e => panic!("Expected FileError::NotFound, got {:?}", e),
        };

        assert_eq!(path, source);

        Ok(())
    }

    #[test]
    fn move_file_raises_error_when_target_already_exist() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let source = dir.path().join("file1.txt");
        fs::write(&source, "File contents")?;

        let dest_dir = dir.path().join("dest");
        fs::create_dir(&dest_dir)?;

        let target = dest_dir.join("target.txt");
        fs::write(&target, "target")?;

        let path = match move_file(&source, &target).unwrap_err() {
            FileError::FileAlreadyExist(file) => file,
            e => panic!("Expected FileError::FileAlreadyExist, got {:?}", e),
        };

        assert_eq!(target, path);

        Ok(())
    }

    #[test]
    fn move_file_raises_error_when_source_is_dir() -> Result<(), FileError> {
        let dir = TempDir::new()?;

        let source = dir.path().join("dir1");
        fs::create_dir(&source)?;

        let dest_file = dir.path().join("test.txt");

        let path = match move_file(&source, &dest_file).unwrap_err() {
            FileError::NotFile(path) => path,
            e => panic!("Expected FileError::NotFound, got {:?}", e),
        };

        assert_eq!(path, source);
        Ok(())
    }
}
