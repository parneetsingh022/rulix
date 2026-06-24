#[cfg(test)]
mod tests {

    use std::{fs, path::PathBuf};
    use undo_fs::{errors::FileError, file_handler::FileHandler};

    fn cross_device_dirs() -> Option<(PathBuf, PathBuf)> {
        let src = std::env::var("MOVE_TEST_SRC").ok()?;
        let dst = std::env::var("MOVE_TEST_DST").ok()?;
        Some((PathBuf::from(src), PathBuf::from(dst)))
    }

    #[test]
    #[ignore = "requires MOVE_TEST_SRC and MOVE_TEST_DST on different filesystems"]
    fn cross_device_move_file_and_undo() -> Result<(), FileError> {
        let (src_dir, dst_dir) =
            cross_device_dirs().expect("MOVE_TEST_SRC and MOVE_TEST_DST must be set");

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
