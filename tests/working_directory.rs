// Integration tests for working_directory resolution.
//
// These tests require sudo access (walltime mode uses systemd-run),
// so they are gated behind the GITHUB_ACTIONS env var.

#[test_with::env(GITHUB_ACTIONS)]
mod tests {
    use assert_cmd::Command;
    use predicates::str::contains;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::sync::Mutex;

    /// Tests use systemd-run/perf which cannot run concurrently.
    static SERIAL: Mutex<()> = Mutex::new(());

    fn codspeed_cmd(cwd: &Path) -> Command {
        let mut cmd = Command::cargo_bin("codspeed").unwrap();
        cmd.current_dir(cwd);
        cmd.env("CODSPEED_TOKEN", "test-token");
        cmd
    }

    /// Create a bash script that validates CWD matches the expected absolute path.
    fn write_cwd_check_script(path: &Path, expected_dir: &Path) {
        let expected = expected_dir.to_string_lossy();
        let content = format!(
            r#"#!/bin/bash
ACTUAL=$(pwd)
EXPECTED="{expected}"
if [ "$ACTUAL" != "$EXPECTED" ]; then
  echo "FAIL: expected $EXPECTED, got $ACTUAL" >&2
  exit 1
fi
"#
        );
        fs::write(path, content).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    /// Write a codspeed.yml config file.
    fn write_config(path: &Path, content: &str) {
        fs::write(path, content).unwrap();
    }

    // -------------------------------------------------------------------
    // Config targets tests (exec targets via exec-harness)
    // -------------------------------------------------------------------

    /// Config targets — root working_directory resolves relative to config dir.
    #[test]
    fn config_targets_root_working_directory_resolves_relative_to_config_dir() {
        let _lock = SERIAL.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let subdir = root.join("subdir");
        fs::create_dir_all(&subdir).unwrap();
        let run_from = root.join("run-from-here");
        fs::create_dir_all(&run_from).unwrap();

        let script = subdir.join("check_cwd.sh");
        write_cwd_check_script(&script, &subdir);

        let config_path = root.join("codspeed.yaml");
        write_config(
            &config_path,
            &format!(
                r#"
options:
  working-directory: subdir
benchmarks:
  - exec: {script}
    options:
      max-rounds: 1
      warmup-time: "0s"
"#,
                script = script.display(),
            ),
        );

        codspeed_cmd(&run_from)
            .args([
                "--config",
                &config_path.to_string_lossy(),
                "run",
                "-m",
                "walltime",
                "--skip-upload",
                "--show-full-output",
            ])
            .assert()
            .success();
    }

    /// Config targets — no working_directory defaults to config dir.
    #[test]
    fn config_targets_no_working_directory_defaults_to_config_dir() {
        let _lock = SERIAL.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let run_from = root.join("run-from-here");
        fs::create_dir_all(&run_from).unwrap();

        let script = root.join("check_cwd.sh");
        write_cwd_check_script(&script, root);

        let config_path = root.join("codspeed.yaml");
        write_config(
            &config_path,
            &format!(
                r#"
benchmarks:
  - exec: {script}
    options:
      max-rounds: 1
      warmup-time: "0s"
"#,
                script = script.display(),
            ),
        );

        codspeed_cmd(&run_from)
            .args([
                "--config",
                &config_path.to_string_lossy(),
                "run",
                "-m",
                "walltime",
                "--skip-upload",
                "--show-full-output",
            ])
            .assert()
            .success();
    }

    /// Config targets — `--working-directory` CLI flag is ignored with warning.
    /// The warning is emitted via eprintln before the logger is initialized,
    /// so we check stderr for it.
    #[test]
    fn config_targets_cli_working_directory_flag_is_ignored_with_warning() {
        let _lock = SERIAL.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let wrong_dir = root.join("wrong-dir");
        fs::create_dir_all(&wrong_dir).unwrap();

        let script = root.join("check_cwd.sh");
        write_cwd_check_script(&script, root);

        let config_path = root.join("codspeed.yaml");
        write_config(
            &config_path,
            &format!(
                r#"
benchmarks:
  - exec: {script}
    options:
      max-rounds: 1
      warmup-time: "0s"
"#,
                script = script.display(),
            ),
        );

        codspeed_cmd(root)
            .args([
                "--config",
                &config_path.to_string_lossy(),
                "run",
                "-m",
                "walltime",
                "--skip-upload",
                "--show-full-output",
                "--working-directory",
                &wrong_dir.to_string_lossy(),
            ])
            .assert()
            .success()
            .stderr(contains("--working-directory flag is ignored"));
    }

    // -------------------------------------------------------------------
    // SingleCommand (codspeed run -- command) tests
    //
    // SingleCommand uses entrypoint mode — plain scripts don't produce
    // walltime results, so we use --allow-empty and check CWD indirectly
    // via script exit code.
    // -------------------------------------------------------------------

    /// SingleCommand — `--working-directory` CLI flag is used.
    #[test]
    fn single_command_cli_working_directory_is_used() {
        let _lock = SERIAL.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let target_dir = root.join("target-dir");
        fs::create_dir_all(&target_dir).unwrap();

        let script = root.join("check_cwd.sh");
        write_cwd_check_script(&script, &target_dir);

        codspeed_cmd(root)
            .args([
                "run",
                "-m",
                "walltime",
                "--skip-upload",
                "--show-full-output",
                "--allow-empty",
                "--working-directory",
                &target_dir.to_string_lossy(),
                "--",
                &script.to_string_lossy(),
            ])
            .assert()
            .success();
    }

    /// SingleCommand — config file's working_directory is NOT used.
    #[test]
    fn single_command_config_working_directory_is_not_used() {
        let _lock = SERIAL.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let wrong_dir = root.join("wrong-dir");
        fs::create_dir_all(&wrong_dir).unwrap();

        let config_path = root.join("codspeed.yaml");
        write_config(
            &config_path,
            r#"
options:
  working-directory: wrong-dir
"#,
        );

        // The script checks that CWD is root (where we launch from), not wrong-dir
        let script = root.join("check_cwd.sh");
        write_cwd_check_script(&script, root);

        codspeed_cmd(root)
            .args([
                "--config",
                &config_path.to_string_lossy(),
                "run",
                "-m",
                "walltime",
                "--skip-upload",
                "--show-full-output",
                "--allow-empty",
                "--",
                &script.to_string_lossy(),
            ])
            .assert()
            .success();
    }

    // -------------------------------------------------------------------
    // Exec command tests (uses exec-harness)
    // -------------------------------------------------------------------

    /// Exec command — `--working-directory` CLI flag is used.
    #[test]
    fn exec_command_cli_working_directory_is_used() {
        let _lock = SERIAL.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let target_dir = root.join("target-dir");
        fs::create_dir_all(&target_dir).unwrap();

        let script = root.join("check_cwd.sh");
        write_cwd_check_script(&script, &target_dir);

        codspeed_cmd(root)
            .args([
                "exec",
                "-m",
                "walltime",
                "--skip-upload",
                "--show-full-output",
                "--working-directory",
                &target_dir.to_string_lossy(),
                "--warmup-time",
                "0s",
                "--max-rounds",
                "1",
                "--",
                &script.to_string_lossy(),
            ])
            .assert()
            .success();
    }

    /// Exec command — config file's working_directory is NOT used.
    #[test]
    fn exec_command_config_working_directory_is_not_used() {
        let _lock = SERIAL.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let wrong_dir = root.join("wrong-dir");
        fs::create_dir_all(&wrong_dir).unwrap();

        let config_path = root.join("codspeed.yaml");
        write_config(
            &config_path,
            r#"
options:
  working-directory: wrong-dir
"#,
        );

        let script = root.join("check_cwd.sh");
        write_cwd_check_script(&script, root);

        codspeed_cmd(root)
            .args([
                "--config",
                &config_path.to_string_lossy(),
                "exec",
                "-m",
                "walltime",
                "--skip-upload",
                "--show-full-output",
                "--warmup-time",
                "0s",
                "--max-rounds",
                "1",
                "--",
                &script.to_string_lossy(),
            ])
            .assert()
            .success();
    }
}
