#[cfg(test)]
mod tests {
    use assert_cmd::Command;

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
            .stderr(predicates::str::contains("file not found: rule_file.yaml"));
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

    #[test]
    fn invalid_file_path_in_target_must_return_path_in_error() {
        let mut cmd = Command::cargo_bin("rulix").unwrap();

        let _ = cmd
            .args(["run", "--rules", "tests/fixtures/invalid_target_path.yaml"])
            .assert()
            .failure()
            .stderr(predicates::str::contains("invalid/file/path"));
    }
}
