#[cfg(test)]
mod tests {
    use std::fs;

    use assert_cmd::Command;
    use tempfile::TempDir;

    #[test]
    fn help_prints_usage() {
        let mut cmd = Command::cargo_bin("rulix").unwrap();

        cmd.arg("--help")
            .assert()
            .success()
            .stdout(predicates::str::contains("Usage"));
    }

    #[test]
    fn prints_current_version_with_version_flag() {
        let current_version = env!("CARGO_PKG_VERSION");

        let mut cmd = Command::cargo_bin("rulix").unwrap();

        cmd.arg("--version")
            .assert()
            .success()
            .stdout(predicates::str::contains(format!(
                "rulix {}",
                current_version
            )));
    }

    // #############################################################################
    // # List command                                                              #
    // #############################################################################

    #[test]
    fn list_succeeds_when_default_config_is_missing() {
        let mut cmd = Command::cargo_bin("rulix").unwrap();

        cmd.arg("list")
            .assert()
            .success()
            .stdout(predicates::str::contains("No rules to show."));
    }

    #[test]
    fn list_returns_error_when_user_provided_config_file_does_not_exist() {
        let mut cmd = Command::cargo_bin("rulix").unwrap();

        cmd.args(["list", "--rules", "rule_file.yaml"])
            .assert()
            .failure()
            .stderr(predicates::str::contains("path not found: rule_file.yaml"));
    }

    #[test]
    fn list_outputs_rules_from_yaml_file() {
        let mut cmd = Command::cargo_bin("rulix").unwrap();

        let output = cmd
            .args(["list", "--rules", "tests/fixtures/move_files_with_ext.yaml"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let stdout = String::from_utf8(output).unwrap();

        insta::assert_snapshot!(stdout);
    }

    // #############################################################################
    // # Run command                                                               #
    // #############################################################################
    macro_rules! create_file {
        ($dir:expr, $name:expr, $contents:expr) => {{
            let path = $dir.join($name);
            std::fs::write(&path, $contents).unwrap();
            path
        }};
    }

    fn create_test_project() -> TempDir {
        let dir = TempDir::new().unwrap();

        let target_dir = dir.path().join("dir1");
        fs::create_dir(&target_dir).unwrap();

        fs::create_dir(dir.path().join("text")).unwrap();
        fs::create_dir(dir.path().join("json")).unwrap();

        create_file!(&target_dir, "txt_file1.txt", "Contents file 1");
        create_file!(&target_dir, "txt_file2.txt", "Contents file 2");
        create_file!(&target_dir, "txt_file3.txt", "Contents file 3");

        create_file!(&target_dir, "json_file1.json", "{name: \"json 1\"}");
        create_file!(&target_dir, "json_file2.json", "{name: \"json 2\"}");
        create_file!(&target_dir, "json_file3.json", "{name: \"json 3\"}");

        create_file!(&target_dir, "rust_file1.rs", "fn rust1() {}");
        create_file!(&target_dir, "rust_file2.rs", "fn rust2() {}");
        create_file!(&target_dir, "rust_file3.rs", "fn rust3() {}");

        create_file!(
            dir.path(),
            "rules.yaml",
            r#"
rules:
  - name: "organize-desktop"
    target: "dir1"
    steps:
      - match:
          ext: "txt"
      - move_to: "text/"

  - name: "new-rule"
    target: "dir1"
    steps:
      - match:
          ext: "json"
      - move_to: "json/"

  - name: "move-python-files"
    target: "dir1"
    steps:
      - match:
          ext: ".py"
      - move_to: "python/"
"#
        );

        dir
    }

    #[test]
    fn run_defaults_to_dry_run() {
        let dir = create_test_project();

        let output = Command::cargo_bin("rulix")
            .unwrap()
            .current_dir(dir.path())
            .args(["run", "--rules", "rules.yaml"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let output = String::from_utf8(output).unwrap();

        insta::assert_snapshot!(output);

        assert!(dir.path().join("dir1/txt_file1.txt").exists());
        assert!(dir.path().join("dir1/txt_file2.txt").exists());
        assert!(dir.path().join("dir1/txt_file3.txt").exists());

        assert!(dir.path().join("dir1/json_file1.json").exists());
        assert!(dir.path().join("dir1/json_file2.json").exists());
        assert!(dir.path().join("dir1/json_file3.json").exists());

        assert!(dir.path().join("dir1/rust_file1.rs").exists());
        assert!(dir.path().join("dir1/rust_file2.rs").exists());
        assert!(dir.path().join("dir1/rust_file3.rs").exists());

        assert!(!dir.path().join("text/txt_file1.txt").exists());
        assert!(!dir.path().join("json/json_file1.json").exists());
    }

    #[test]
    fn run_with_execute_moves_matching_files() {
        let dir = create_test_project();

        let _ = Command::cargo_bin("rulix")
            .unwrap()
            .current_dir(dir.path())
            .args(["run", "--rules", "rules.yaml", "--execute"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        assert!(!dir.path().join("dir1/txt_file1.txt").exists());
        assert!(!dir.path().join("dir1/txt_file2.txt").exists());
        assert!(!dir.path().join("dir1/txt_file3.txt").exists());

        assert!(!dir.path().join("dir1/json_file1.json").exists());
        assert!(!dir.path().join("dir1/json_file2.json").exists());
        assert!(!dir.path().join("dir1/json_file3.json").exists());

        assert!(dir.path().join("text/txt_file1.txt").exists());
        assert!(dir.path().join("text/txt_file2.txt").exists());
        assert!(dir.path().join("text/txt_file3.txt").exists());

        assert!(dir.path().join("json/json_file1.json").exists());
        assert!(dir.path().join("json/json_file2.json").exists());
        assert!(dir.path().join("json/json_file3.json").exists());

        assert!(dir.path().join("dir1/rust_file1.rs").exists());
        assert!(dir.path().join("dir1/rust_file2.rs").exists());
        assert!(dir.path().join("dir1/rust_file3.rs").exists());
    }

    #[test]
    fn invalid_file_path_in_target_must_return_path_in_error() {
        let mut cmd = Command::cargo_bin("rulix").unwrap();

        let _ = cmd
            .args([
                "run",
                "--rules",
                "tests/fixtures/invalid_target_path.yaml",
                "--execute",
            ])
            .assert()
            .failure()
            .stderr(predicates::str::contains("invalid/file/path"));
    }

    #[test]
    fn notify_step_produces_right_output() {
        let mut cmd = Command::cargo_bin("rulix").unwrap();

        let output = cmd
            .args([
                "run",
                "--rules",
                "tests/fixtures/notify_step.yaml",
                "--execute",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let stdout = String::from_utf8(output).unwrap();

        insta::assert_snapshot!(stdout);
    }
}
