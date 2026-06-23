//! Cryptographic hashing utilities for the filesystem.
//!
//! This module provides functions to safely generate integrity verification
//! checksums for files on disk, ensuring compatibility with filesystem
//! rollbacks.

use crate::errors::FileError;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

/// Computes the SHA-256 checksum of a file located at the specified path.
///
/// This function reads the file sequentially in 8KB chunks. This streaming
/// approach keeps the memory footprint extremely low and constant, preventing
/// out-of-memory crashes on very large files.
///
/// # Arguments
///
/// * `path` - A reference or type that can be treated as a filesystem path (`AsRef<Path>`).
///
/// # Returns
///
/// * `Ok(String)` - A 64-character, lower-case hexadecimal encoded SHA-256 hash string.
/// * `Err(FileError)` - An error wrapping underlying physical file system issues.
///
/// # Examples
///
/// ```rust
/// # use std::fs;
/// # // 1. Setup a dynamic file that exists during the doc-test execution
/// # let test_file = "sample_doc_test.txt";
/// # fs::write(test_file, "Hello, doc-test!").unwrap();
/// #
/// use undo_fs::checksum::get_file_hash;
///
/// let hash = get_file_hash(test_file).unwrap();
/// assert_eq!(hash.len(), 64);
///
/// # // 2. Clean up after the test finishes so we don't pollute the directory
/// # fs::remove_file(test_file).unwrap();
/// ```
///
pub fn get_file_hash(path: impl AsRef<Path>) -> Result<String, FileError> {
    let mut file = match File::open(&path) {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Err(FileError::NotFound(path.as_ref().to_path_buf()));
        }
        Err(err) => {
            return Err(FileError::Io(err));
        }
    };

    let mut hasher = Sha256::new();

    // 8KB buffer to read file data in chunks
    let mut buffer = [0u8; 1024 * 8];

    loop {
        let n = file.read(&mut buffer)?;

        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n])
    }

    let result = hasher.finalize();

    Ok(hex::encode(result))
}

/// Verifies that a file's current contents match the expected checksum.
///
/// The file is hashed and compared against `checksum`. Returns `true` if the
/// computed hash matches the expected value and `false` otherwise.
///
/// This function is typically used to ensure that a file has not been modified
/// since an operation was recorded, allowing potentially destructive actions
/// such as undo operations to be performed safely.
///
/// # Errors
///
/// Returns [`FileError`] if the file cannot be accessed or its hash cannot be
/// computed.
pub fn file_checksum_matches(path: impl AsRef<Path>, checksum: &str) -> Result<bool, FileError> {
    let actual = get_file_hash(path)?;

    Ok(actual == checksum)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_hash_empty_file() -> Result<(), FileError> {
        let dir = TempDir::new()?;
        let file_path = dir.path().join("empty.txt");

        fs::write(&file_path, b"")?;

        let hash = get_file_hash(&file_path)?;

        let expected_empty_sha256 =
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(hash, expected_empty_sha256);

        Ok(())
    }

    #[test]
    fn test_hash_small_file() -> Result<(), FileError> {
        let dir = TempDir::new()?;
        let file_path = dir.path().join("small.txt");

        fs::write(&file_path, b"Hello, undo-fs!")?;

        let hash = get_file_hash(&file_path)?;

        assert_eq!(hash.len(), 64); // SHA-256 hex strings are exactly 64 characters

        // Ensure running it twice yields the exact same result (determinism)
        let hash_repeat = get_file_hash(&file_path)?;
        assert_eq!(hash, hash_repeat);

        Ok(())
    }

    #[test]
    fn test_hash_large_file_crossing_buffers() -> Result<(), FileError> {
        let dir = TempDir::new()?;
        let file_path = dir.path().join("large.bin");

        // Generate exactly 20KB of repeating data (forces our 8KB buffer to loop multiple times)
        let mut large_data = Vec::with_capacity(1024 * 20);
        for i in 0..(1024 * 20) {
            large_data.push((i % 256) as u8);
        }
        fs::write(&file_path, &large_data)?;

        let hash = get_file_hash(&file_path)?;
        assert_eq!(hash.len(), 64);

        // Modifying even a single trailing byte must completely alter the hash result
        large_data[1024 * 20 - 1] ^= 1;
        fs::write(&file_path, &large_data)?;

        let altered_hash = get_file_hash(&file_path)?;
        assert_ne!(hash, altered_hash);

        Ok(())
    }

    #[test]
    fn test_hash_file_not_found() {
        let non_existent_path = Path::new("this_file_definitely_does_not_exist.xyz");

        let result = get_file_hash(non_existent_path);

        // Verify it bubbles up a clean error variant instead of panicking
        assert!(result.is_err());
        assert!(matches!(result.err(), Some(FileError::NotFound(_))))
    }

    #[test]
    fn verify_file_checksum_returns_true_when_checksum_matches() {
        let path = std::env::temp_dir().join("checksum_match_test.txt");

        fs::write(&path, "hello world").unwrap();

        let checksum = get_file_hash(&path).unwrap();

        let result = file_checksum_matches(&path, &checksum).unwrap();

        assert!(result);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn verify_file_checksum_returns_false_when_checksum_does_not_match() {
        let path = std::env::temp_dir().join("checksum_mismatch_test.txt");

        fs::write(&path, "hello world").unwrap();

        let result = file_checksum_matches(&path, "not-the-real-checksum").unwrap();

        assert!(!result);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn verify_file_checksum_detects_modified_file_contents() {
        let path = std::env::temp_dir().join("checksum_modified_test.txt");

        fs::write(&path, "original contents").unwrap();

        let original_checksum = get_file_hash(&path).unwrap();

        fs::write(&path, "modified contents").unwrap();

        let result = file_checksum_matches(&path, &original_checksum).unwrap();

        assert!(!result);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn verify_file_checksum_returns_error_for_missing_file() {
        let path = std::env::temp_dir().join("checksum_missing_file_test.txt");

        let result = file_checksum_matches(&path, "some-checksum");

        assert!(result.is_err());
    }
}
