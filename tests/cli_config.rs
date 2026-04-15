use clap::Parser;
use foreman::app::NotificationProfile;
use foreman::cli::{run, Cli, RunOutcome};
use foreman::doctor::DoctorReport;
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

#[test]
fn repo_flag_requires_setup_or_doctor() {
    let cli = Cli::parse_from(["foreman", "--repo", "/tmp/elsewhere"]);

    let error = run(cli).expect_err("repo should be rejected outside setup/doctor");

    assert_eq!(
        error.to_string(),
        "--repo is only supported with --setup or --doctor"
    );
}

#[test]
fn doctor_json_reports_repo_findings() {
    let temp_dir = tempdir().expect("temp dir should exist");
    let home_dir = tempdir().expect("home dir should exist");
    std::fs::create_dir_all(temp_dir.path().join(".git")).expect("git dir should exist");

    let output = Command::new(foreman_bin())
        .args([
            "--doctor",
            "--doctor-json",
            "--config-file",
            temp_dir
                .path()
                .join("config.toml")
                .to_str()
                .expect("utf-8 path"),
            "--log-dir",
            temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        ])
        .current_dir(temp_dir.path())
        .env("HOME", home_dir.path())
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let report: DoctorReport = serde_json::from_str(&stdout).expect("report should parse");
    let reported_repo = report
        .repo_path
        .as_deref()
        .expect("report should include inferred repo path")
        .canonicalize()
        .expect("reported repo path should canonicalize");
    let expected_repo = temp_dir
        .path()
        .canonicalize()
        .expect("temp repo path should canonicalize");
    assert_eq!(reported_repo, expected_repo);
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.id == "codex-hook-file-missing"));
}

#[test]
fn doctor_handles_invalid_config_without_exiting_early() {
    let temp_dir = tempdir().expect("temp dir should exist");
    let home_dir = tempdir().expect("home dir should exist");
    std::fs::create_dir_all(temp_dir.path().join(".git")).expect("git dir should exist");
    let config_file = temp_dir.path().join("config.toml");
    std::fs::write(&config_file, "[notifications\nenabled = true\n")
        .expect("config should be written");

    let output = Command::new(foreman_bin())
        .args([
            "--doctor",
            "--doctor-json",
            "--repo",
            temp_dir.path().to_str().expect("utf-8 path"),
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        ])
        .env("HOME", home_dir.path())
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let report: DoctorReport = serde_json::from_str(&stdout).expect("report should parse");
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.summary.contains("could not be parsed")));
}

#[test]
fn setup_handles_invalid_config_without_exiting_early() {
    let temp_dir = tempdir().expect("temp dir should exist");
    std::fs::create_dir_all(temp_dir.path().join(".git")).expect("git dir should exist");
    let config_file = temp_dir.path().join("config.toml");
    let invalid_config = "[notifications\nenabled = true\n";
    std::fs::write(&config_file, invalid_config).expect("config should be written");

    let output = Command::new(foreman_bin())
        .args([
            "--setup",
            "--project",
            "--codex",
            "--repo",
            temp_dir.path().to_str().expect("utf-8 path"),
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        ])
        .output()
        .expect("command should run");

    assert!(output.status.success());
    assert!(temp_dir.path().join(".codex/hooks.json").exists());
    assert_eq!(
        std::fs::read_to_string(&config_file).expect("config should still exist"),
        invalid_config
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("could not be parsed"));
}

#[test]
fn doctor_strict_fails_when_repo_has_blocking_findings() {
    let temp_dir = tempdir().expect("temp dir should exist");
    let home_dir = tempdir().expect("home dir should exist");
    std::fs::create_dir_all(temp_dir.path().join(".git")).expect("git dir should exist");
    std::fs::create_dir_all(temp_dir.path().join(".codex")).expect("codex dir should exist");
    std::fs::write(temp_dir.path().join(".codex/hooks.json"), "{not-json")
        .expect("hooks file should be written");

    let output = Command::new(foreman_bin())
        .args([
            "--doctor",
            "--doctor-strict",
            "--repo",
            temp_dir.path().to_str().expect("utf-8 path"),
            "--config-file",
            temp_dir
                .path()
                .join("config.toml")
                .to_str()
                .expect("utf-8 path"),
            "--log-dir",
            temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        ])
        .env("HOME", home_dir.path())
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(stdout.contains("Codex hook config could not be parsed"));
    assert!(stderr.contains("doctor found"));
}

#[test]
fn help_surfaces_setup_first_run_flow() {
    let output = Command::new(foreman_bin())
        .arg("--help")
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("First-time setup:"));
    assert!(stdout.contains("foreman --setup --user --project"));
    assert!(stdout.contains("foreman --doctor"));
    assert!(stdout.contains("foreman --setup --project --codex --repo /path/to/repo"));
    assert!(stdout.contains("--repo <REPO>"));
    assert!(!stdout.contains("--doctor-fix"));
}

#[test]
fn setup_dry_run_previews_safe_changes_without_writing() {
    let temp_dir = tempdir().expect("temp dir should exist");
    std::fs::create_dir_all(temp_dir.path().join(".git")).expect("git dir should exist");
    let config_file = temp_dir.path().join("config.toml");

    let output = Command::new(foreman_bin())
        .args([
            "--setup",
            "--dry-run",
            "--repo",
            temp_dir.path().to_str().expect("utf-8 path"),
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        ])
        .output()
        .expect("command should run");

    assert!(output.status.success());
    assert!(!config_file.exists());
    assert!(!temp_dir.path().join(".codex/hooks.json").exists());
    assert!(!temp_dir.path().join(".pi/extensions/foreman.ts").exists());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Foreman setup preview"));
    assert!(stdout.contains("Targets: project"));
    assert!(stdout.contains("Providers: Claude Code, Codex CLI, Pi"));
    assert!(stdout.contains("Planned changes"));
    assert!(stdout.contains("foreman --setup --project --repo"));
    assert!(stdout.contains("foreman --doctor --repo"));
}

#[test]
fn setup_writes_safe_repo_files() {
    let temp_dir = tempdir().expect("temp dir should exist");
    std::fs::create_dir_all(temp_dir.path().join(".git")).expect("git dir should exist");
    let config_file = temp_dir.path().join("config.toml");

    let output = Command::new(foreman_bin())
        .args([
            "--setup",
            "--repo",
            temp_dir.path().to_str().expect("utf-8 path"),
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        ])
        .output()
        .expect("command should run");

    assert!(output.status.success());
    assert!(config_file.exists());
    assert!(temp_dir.path().join(".codex/hooks.json").exists());
    assert!(temp_dir.path().join(".pi/extensions/foreman.ts").exists());
    assert!(temp_dir.path().join(".claude/settings.local.json").exists());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Foreman setup"));
    assert!(stdout.contains("Applied changes"));
}

#[test]
fn setup_user_writes_global_provider_files() {
    let temp_dir = tempdir().expect("temp dir should exist");
    std::fs::create_dir_all(temp_dir.path().join(".git")).expect("git dir should exist");
    let home_dir = tempdir().expect("home dir should exist");
    let config_file = temp_dir.path().join("config.toml");

    let output = Command::new(foreman_bin())
        .args([
            "--setup",
            "--user",
            "--repo",
            temp_dir.path().to_str().expect("utf-8 path"),
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        ])
        .env("HOME", home_dir.path())
        .output()
        .expect("command should run");

    assert!(output.status.success());
    assert!(home_dir.path().join(".claude/settings.local.json").exists());
    assert!(home_dir.path().join(".codex/hooks.json").exists());
    assert!(home_dir.path().join(".pi/extensions/foreman.ts").exists());
    assert!(!temp_dir.path().join(".codex/hooks.json").exists());
    assert!(!temp_dir.path().join(".pi/extensions/foreman.ts").exists());
}

#[test]
fn setup_can_limit_writes_to_one_project_provider() {
    let temp_dir = tempdir().expect("temp dir should exist");
    std::fs::create_dir_all(temp_dir.path().join(".git")).expect("git dir should exist");
    let config_file = temp_dir.path().join("config.toml");

    let output = Command::new(foreman_bin())
        .args([
            "--setup",
            "--project",
            "--codex",
            "--repo",
            temp_dir.path().to_str().expect("utf-8 path"),
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        ])
        .output()
        .expect("command should run");

    assert!(output.status.success());
    assert!(temp_dir.path().join(".codex/hooks.json").exists());
    assert!(!temp_dir.path().join(".claude/settings.local.json").exists());
    assert!(!temp_dir.path().join(".pi/extensions/foreman.ts").exists());
    assert!(temp_dir.path().join("codex-native").exists());
    assert!(!temp_dir.path().join("claude-native").exists());
    assert!(!temp_dir.path().join("pi-native").exists());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("Targets: project"));
    assert!(stdout.contains("Providers: Codex CLI"));
}

#[test]
fn setup_user_outside_repo_reports_user_scope() {
    let temp_dir = tempdir().expect("temp dir should exist");
    let home_dir = tempdir().expect("home dir should exist");

    let output = Command::new(foreman_bin())
        .args([
            "--setup",
            "--user",
            "--config-file",
            temp_dir
                .path()
                .join("config.toml")
                .to_str()
                .expect("utf-8 path"),
            "--log-dir",
            temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        ])
        .current_dir(temp_dir.path())
        .env("HOME", home_dir.path())
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout
        .contains("Repo: none detected from the current directory; applying user-scoped setup."));
}
