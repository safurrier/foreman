use std::process::Command;
use tempfile::tempdir;

fn foreman_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman")
}

#[test]
fn binary_bootstrap_logs_system_stats_and_tmux_failure_alert() {
    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let missing_socket = config_dir.path().join("missing.sock");

    let output = Command::new(foreman_bin())
        .arg("--bootstrap-only")
        .args(["--config-file"])
        .arg(config_dir.path().join("config.toml"))
        .args(["--log-dir"])
        .arg(log_dir.path())
        .args(["--tmux-socket"])
        .arg(&missing_socket)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Foreman bootstrap complete."));

    let log_contents =
        std::fs::read_to_string(log_dir.path().join("latest.log")).expect("log should exist");
    assert!(log_contents.contains("system_stats_snapshot"));
    assert!(log_contents.contains("cpu_pressure="));
    assert!(log_contents.contains("memory_pressure="));
    assert!(log_contents.contains("tmux_bootstrap_error"));
    assert!(log_contents.contains("operator_alert"));
    assert!(log_contents.contains("source=tmux"));
    assert!(log_contents.contains("level=warn"));
}
