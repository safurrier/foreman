use clap::Parser;
use foreman::app::NotificationProfile;
use foreman::cli::{run, Cli, RunOutcome};
use std::process::Command;
use tempfile::tempdir;

fn foreman_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman")
}

#[test]
fn config_path_uses_foreman_config_home_environment() {
    let temp_dir = tempdir().expect("temp dir should exist");

    let output = Command::new(foreman_bin())
        .arg("--config-path")
        .env("FOREMAN_CONFIG_HOME", temp_dir.path())
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert_eq!(
        stdout.trim(),
        temp_dir
            .path()
            .join("foreman/config.toml")
            .display()
            .to_string()
    );
}

#[test]
fn init_config_creates_default_file() {
    let temp_dir = tempdir().expect("temp dir should exist");
    let config_file = temp_dir.path().join("custom-config.toml");

    let output = Command::new(foreman_bin())
        .args(["--init-config", "--config-file"])
        .arg(&config_file)
        .output()
        .expect("command should run");

    assert!(output.status.success());
    assert!(config_file.exists());

    let config_contents =
        std::fs::read_to_string(config_file).expect("config file should be readable");
    assert!(config_contents.contains("[monitoring]"));
    assert!(config_contents.contains("poll_interval_ms"));
}

#[test]
fn default_run_creates_run_log_and_latest_log() {
    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");

    let output = Command::new(foreman_bin())
        .arg("--bootstrap-only")
        .env("FOREMAN_CONFIG_HOME", config_dir.path())
        .env("FOREMAN_LOG_DIR", log_dir.path())
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Foreman bootstrap complete."));

    let latest_log = log_dir.path().join("latest.log");
    assert!(latest_log.exists());

    let log_contents = std::fs::read_to_string(latest_log).expect("latest log should be readable");
    assert!(log_contents.contains("bootstrap_complete"));
    assert!(!log_contents.contains("[DEBUG]"));
}

#[test]
fn debug_flag_emits_debug_log_lines() {
    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");

    let output = Command::new(foreman_bin())
        .args(["--bootstrap-only", "--debug"])
        .env("FOREMAN_CONFIG_HOME", config_dir.path())
        .env("FOREMAN_LOG_DIR", log_dir.path())
        .output()
        .expect("command should run");

    assert!(output.status.success());

    let log_contents =
        std::fs::read_to_string(log_dir.path().join("latest.log")).expect("log should exist");
    assert!(log_contents.contains("[DEBUG] bootstrap_debug_logging_enabled"));
}

#[test]
fn bootstrap_uses_notification_defaults_from_config() {
    let temp_dir = tempdir().expect("temp dir should exist");
    let config_file = temp_dir.path().join("config.toml");
    std::fs::write(
        &config_file,
        r#"
[notifications]
enabled = true
cooldown_ticks = 7
active_profile = "attention-only"
"#,
    )
    .expect("config should be written");

    let cli = Cli::parse_from([
        "foreman",
        "--config-file",
        config_file.to_str().expect("utf-8 path"),
        "--log-dir",
        temp_dir.path().join("logs").to_str().expect("utf-8 path"),
    ]);

    let outcome = run(cli).expect("bootstrap should succeed");
    let summary = match outcome {
        RunOutcome::Bootstrapped(summary) => summary,
        other => panic!("expected bootstrapped outcome, got {other:?}"),
    };

    assert_eq!(
        summary.state.notifications.profile,
        NotificationProfile::AttentionOnly
    );
    assert_eq!(summary.state.notifications.cooldown_ticks, 7);
}
