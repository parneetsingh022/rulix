use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{errors::FileError, steps::MatchCriteria};

pub fn execute(
    target: &Path,
    criteria: &MatchCriteria,
    matched_files: &mut Vec<PathBuf>,
) -> Result<(), FileError> {
    if !target.exists() {
        return Err(FileError::NotFound(target.to_path_buf()));
    }

    // Clear previously filtered files to support nested matching pipelines.
    // A single rule may chain multiple actions after a match step, followed by
    // subsequent match steps to filter new subsets of files. Clearing the
    // `matched_files` vector retains its allocated capacity, avoiding
    // reallocation overhead during these sequential operations.
    matched_files.clear();

    for entry in fs::read_dir(target)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.is_file() && criteria.matches(file_path.as_path()) {
            matched_files.push(file_path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn match_execute_returns_not_found_when_target_does_not_exist() {
        let temp_dir = tempdir().unwrap();

        let missing_path = temp_dir.path().join("does-not-exist");

        let criteria = MatchCriteria {
            ext: Some("txt".to_string()),
        };

        let mut matched_files = Vec::new();

        let err = execute(&missing_path, &criteria, &mut matched_files).unwrap_err();

        let error_message = err.to_string();

        assert!(error_message.contains("does-not-exist"));
        assert!(matched_files.is_empty());
    }

    #[test]
    fn match_execute_returns_file_with_extension_containing_leading_dot() {
        let temp_dir = tempdir().unwrap();
        let text_file = temp_dir.path().join("file.txt");
        std::fs::write(&text_file, "hello").unwrap();

        let criteria = MatchCriteria {
            ext: Some(".txt".to_string()),
        };

        let mut matched_files: Vec<PathBuf> = Vec::new();

        execute(temp_dir.path(), &criteria, &mut matched_files).unwrap();

        assert_eq!(matched_files, vec![text_file]);
    }

    #[test]
    fn match_execute_adds_only_files_matching_extension() {
        let temp_dir = tempdir().unwrap();

        let txt_file = temp_dir.path().join("hello.txt");
        let another_txt_file = temp_dir.path().join("notes.txt");
        let rs_file = temp_dir.path().join("main.rs");
        let extensionless_file = temp_dir.path().join("README");
        let txt_dir = temp_dir.path().join("folder.txt");

        std::fs::write(&txt_file, "hello").unwrap();
        std::fs::write(&another_txt_file, "notes").unwrap();
        std::fs::write(&rs_file, "fn main() {}").unwrap();
        std::fs::write(&extensionless_file, "readme").unwrap();
        std::fs::create_dir(&txt_dir).unwrap();

        let nested_txt_file = txt_dir.join("nested.txt");
        std::fs::write(&nested_txt_file, "nested").unwrap();

        let criteria = MatchCriteria {
            ext: Some("txt".to_string()),
        };

        let mut matched_files = Vec::new();

        execute(temp_dir.path(), &criteria, &mut matched_files).unwrap();

        matched_files.sort();

        let mut expected = vec![txt_file, another_txt_file];
        expected.sort();

        assert_eq!(matched_files, expected);
    }
}
