#[cfg(test)]
mod tests {
    use assert_cmd::Command;

    #[test]
    fn notify_step_produces_right_output() {
        let mut cmd = Command::cargo_bin("rulix").unwrap();

        let output = cmd
            .args(["run", "--rules", "tests/fixtures/notify_step.yaml"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let stdout = String::from_utf8(output).unwrap();

        insta::assert_snapshot!(stdout);
    }
}
