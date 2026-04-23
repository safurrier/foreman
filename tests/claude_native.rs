mod support;

use clap::Parser;
use foreman::app::{AgentStatus, IntegrationMode, SelectionTarget};
use foreman::cli::{run, Cli, RunOutcome};
use support::tmux::TmuxFixture;
use tempfile::tempdir;

fn foreman_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman")
}

#[test]
fn bootstrap_prefers_claude_native_signal_and_falls_back_to_compatibility() {
    let fixture = TmuxFixture::new();
    let pane_id = fixture.new_session(
        "alpha",
        &fixture.shell_command("Claude Code is thinking about the patch"),
    );
    fixture.wait_for_capture(&pane_id, "Claude Code is thinking");

    let temp_dir = tempdir().expect("temp dir should exist");
    let native_dir = temp_dir.path().join("native");
    std::fs::create_dir_all(&native_dir).expect("native dir should exist");
    std::fs::write(
        native_dir.join(format!("{pane_id}.json")),
        r#"{"status":"needs_attention","activity_score":95}"#,
    )
    .expect("native signal should exist");

    let cli = Cli::parse_from([
        "foreman",
        "--config-file",
        temp_dir
            .path()
            .join("config.toml")
            .to_str()
            .expect("utf-8 path"),
        "--log-dir",
        temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        "--tmux-socket",
        fixture.socket_path().to_str().expect("utf-8 path"),
        "--claude-native-dir",
        native_dir.to_str().expect("utf-8 path"),
    ]);

    let first = match run(cli.clone()).expect("bootstrap should succeed") {
        RunOutcome::Bootstrapped(summary) => summary,
        other => panic!("expected bootstrapped outcome, got {other:?}"),
    };

    let pane = match first
        .state
        .selection
        .as_ref()
        .expect("selection should exist")
    {
        SelectionTarget::Session(session_id) => first
            .state
            .inventory
            .session(session_id)
            .and_then(|session| session.windows.first())
            .and_then(|window| window.panes.first())
            .expect("pane should exist"),
        other => panic!("expected session selection, got {other:?}"),
    };
    let agent = pane.agent.as_ref().expect("agent should exist");
    assert_eq!(agent.integration_mode, IntegrationMode::Native);
    assert_eq!(agent.status, AgentStatus::NeedsAttention);
    assert_eq!(first.claude_native.applied, 1);
    assert_eq!(first.claude_native.fallback_to_compatibility, 0);

    std::fs::remove_file(native_dir.join(format!("{pane_id}.json")))
        .expect("native signal should be removable");

    let second = match run(cli).expect("bootstrap should succeed") {
        RunOutcome::Bootstrapped(summary) => summary,
        other => panic!("expected bootstrapped outcome, got {other:?}"),
    };

    let pane = match second
        .state
        .selection
        .as_ref()
        .expect("selection should exist")
    {
        SelectionTarget::Session(session_id) => second
            .state
            .inventory
            .session(session_id)
            .and_then(|session| session.windows.first())
            .and_then(|window| window.panes.first())
            .expect("pane should exist"),
        other => panic!("expected session selection, got {other:?}"),
    };
    let agent = pane.agent.as_ref().expect("agent should exist");
    assert_eq!(agent.integration_mode, IntegrationMode::Compatibility);
    assert_eq!(agent.status, AgentStatus::Working);
    assert_eq!(second.claude_native.applied, 0);
    assert_eq!(second.claude_native.fallback_to_compatibility, 1);
}

#[test]
fn config_can_force_compatibility_even_when_native_signal_exists() {
    let fixture = TmuxFixture::new();
    let pane_id = fixture.new_session(
        "alpha",
        &fixture.shell_command("Claude Code is thinking about the patch"),
    );
    fixture.wait_for_capture(&pane_id, "Claude Code is thinking");

    let temp_dir = tempdir().expect("temp dir should exist");
    let config_file = temp_dir.path().join("config.toml");
    std::fs::write(
        &config_file,
        r#"
[integrations.claude_code]
mode = "compatibility"
"#,
    )
    .expect("config should be written");

    let native_dir = temp_dir.path().join("native");
    std::fs::create_dir_all(&native_dir).expect("native dir should exist");
    std::fs::write(
        native_dir.join(format!("{pane_id}.json")),
        r#"{"status":"needs_attention","activity_score":95}"#,
    )
    .expect("native signal should exist");

    let cli = Cli::parse_from([
        "foreman",
        "--config-file",
        config_file.to_str().expect("utf-8 path"),
        "--log-dir",
        temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        "--tmux-socket",
        fixture.socket_path().to_str().expect("utf-8 path"),
        "--claude-native-dir",
        native_dir.to_str().expect("utf-8 path"),
    ]);

    let summary = match run(cli).expect("bootstrap should succeed") {
        RunOutcome::Bootstrapped(summary) => summary,
        other => panic!("expected bootstrapped outcome, got {other:?}"),
    };

    let pane = match summary
        .state
        .selection
        .as_ref()
        .expect("selection should exist")
    {
        SelectionTarget::Session(session_id) => summary
            .state
            .inventory
            .session(session_id)
            .and_then(|session| session.windows.first())
            .and_then(|window| window.panes.first())
            .expect("pane should exist"),
        other => panic!("expected session selection, got {other:?}"),
    };
    let agent = pane.agent.as_ref().expect("agent should exist");
    assert_eq!(agent.integration_mode, IntegrationMode::Compatibility);
    assert_eq!(agent.status, AgentStatus::Working);
    assert_eq!(summary.claude_native.applied, 0);
    assert_eq!(summary.claude_native.fallback_to_compatibility, 0);
}

#[test]
fn native_preference_falls_back_when_native_source_is_unavailable() {
    let fixture = TmuxFixture::new();
    let pane_id = fixture.new_session(
        "alpha",
        &fixture.shell_command("Claude Code is thinking about the patch"),
    );
    fixture.wait_for_capture(&pane_id, "Claude Code is thinking");

    let temp_dir = tempdir().expect("temp dir should exist");
    let config_file = temp_dir.path().join("config.toml");
    std::fs::write(
        &config_file,
        r#"
[integrations.claude_code]
mode = "native"
"#,
    )
    .expect("config should be written");

    let cli = Cli::parse_from([
        "foreman",
        "--config-file",
        config_file.to_str().expect("utf-8 path"),
        "--log-dir",
        temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        "--tmux-socket",
        fixture.socket_path().to_str().expect("utf-8 path"),
    ]);

    let summary = match run(cli).expect("bootstrap should succeed") {
        RunOutcome::Bootstrapped(summary) => summary,
        other => panic!("expected bootstrapped outcome, got {other:?}"),
    };

    let pane = match summary
        .state
        .selection
        .as_ref()
        .expect("selection should exist")
    {
        SelectionTarget::Session(session_id) => summary
            .state
            .inventory
            .session(session_id)
            .and_then(|session| session.windows.first())
            .and_then(|window| window.panes.first())
            .expect("pane should exist"),
        other => panic!("expected session selection, got {other:?}"),
    };
    let agent = pane.agent.as_ref().expect("agent should exist");
    assert_eq!(agent.integration_mode, IntegrationMode::Compatibility);
    assert_eq!(agent.status, AgentStatus::Working);
    assert_eq!(summary.claude_native.applied, 0);
    assert_eq!(summary.claude_native.fallback_to_compatibility, 1);
    assert!(summary.claude_native.warnings.is_empty());
}

#[test]
fn binary_bootstrap_logs_claude_native_summary() {
    let fixture = TmuxFixture::new();
    let pane_id = fixture.new_session(
        "alpha",
        &fixture.shell_command("Claude Code is thinking about the patch"),
    );
    fixture.wait_for_capture(&pane_id, "Claude Code is thinking");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let native_dir = tempdir().expect("native dir should exist");
    std::fs::write(
        native_dir.path().join(format!("{pane_id}.json")),
        r#"{"status":"idle","activity_score":44}"#,
    )
    .expect("native signal should exist");

    let output = std::process::Command::new(foreman_bin())
        .arg("--bootstrap-only")
        .args(["--config-file"])
        .arg(config_dir.path().join("config.toml"))
        .args(["--log-dir"])
        .arg(log_dir.path())
        .args(["--tmux-socket"])
        .arg(fixture.socket_path())
        .args(["--claude-native-dir"])
        .arg(native_dir.path())
        .output()
        .expect("command should run");

    assert!(output.status.success());

    let log_contents =
        std::fs::read_to_string(log_dir.path().join("latest.log")).expect("log should exist");
    assert!(log_contents.contains("claude_native_summary"));
    assert!(log_contents.contains("applied=1"));
    assert!(log_contents.contains("fallback_to_compatibility=0"));
}
