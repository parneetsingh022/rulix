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

    #[test]
    fn run_defaults_to_dry_run() {
        let dir = TempDir::new().unwrap();

        let target_dir = dir.path().join("dir1");
        fs::create_dir(&target_dir).unwrap();

        fs::create_dir(dir.path().join("text")).unwrap();
        fs::create_dir(dir.path().join("json")).unwrap();

        let txt_file1 = create_file!(&target_dir, "txt_file1.txt", "Contents file 1");
        let txt_file2 = create_file!(&target_dir, "txt_file2.txt", "Contents file 2");
        let txt_file3 = create_file!(&target_dir, "txt_file3.txt", "Contents file 3");

        let json_file1 = create_file!(&target_dir, "json_file1.json", "{name: \"json 1\"}");
        let json_file2 = create_file!(&target_dir, "json_file2.json", "{name: \"json 2\"}");
        let json_file3 = create_file!(&target_dir, "json_file3.json", "{name: \"json 3\"}");

        let rust_file1 = create_file!(&target_dir, "rust_file1.rs", "fn rust1() {}");
        let rust_file2 = create_file!(&target_dir, "rust_file2.rs", "fn rust2() {}");
        let rust_file3 = create_file!(&target_dir, "rust_file3.rs", "fn rust3() {}");

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
"#
        );

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

        assert!(txt_file1.exists());
        assert!(txt_file2.exists());
        assert!(txt_file3.exists());

        assert!(json_file1.exists());
        assert!(json_file2.exists());
        assert!(json_file3.exists());

        assert!(rust_file1.exists());
        assert!(rust_file2.exists());
        assert!(rust_file3.exists());

        assert!(!dir.path().join("test/txt_file1.txt").exists());
        assert!(!dir.path().join("test_files/json_file1.json").exists());
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
