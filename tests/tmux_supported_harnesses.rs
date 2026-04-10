mod support;

use clap::Parser;
use foreman::app::{AgentStatus, HarnessKind, IntegrationMode};
use foreman::cli::{run, Cli, RunOutcome};
use support::tmux::TmuxFixture;
use tempfile::tempdir;

#[test]
fn bootstrap_with_real_tmux_recognizes_supported_harness_matrix() {
    let fixture = TmuxFixture::new();
    let claude_pane = fixture.new_session("claude", &fixture.shell_command("Claude Code ready"));
    let codex_pane = fixture.new_session(
        "codex",
        &fixture.shell_command("Codex CLI waiting for your input"),
    );
    let pi_pane = fixture.new_session("pi", &fixture.shell_command("Pi ready"));
    let gemini_pane = fixture.new_session(
        "gemini",
        &fixture.shell_command("Gemini CLI ready for the next task"),
    );
    let opencode_pane = fixture.new_session(
        "opencode",
        &fixture.shell_command("OpenCode exception: transport failed"),
    );

    fixture.wait_for_capture(&claude_pane, "Claude Code ready");
    fixture.wait_for_capture(&codex_pane, "Codex CLI waiting for your input");
    fixture.wait_for_capture(&pi_pane, "Pi ready");
    fixture.wait_for_capture(&gemini_pane, "Gemini CLI ready for the next task");
    fixture.wait_for_capture(&opencode_pane, "OpenCode exception: transport failed");

    let temp_dir = tempdir().expect("temp dir should exist");
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
    ]);

    let summary = match run(cli).expect("bootstrap should succeed") {
        RunOutcome::Bootstrapped(summary) => summary,
        other => panic!("expected bootstrapped outcome, got {other:?}"),
    };

    assert_eq!(summary.inventory.total_sessions, 5);
    assert_eq!(summary.inventory.total_panes, 5);
    assert_eq!(summary.inventory.visible_sessions, 5);
    assert_eq!(summary.inventory.visible_panes, 5);

    let cases = [
        (
            "claude",
            HarnessKind::ClaudeCode,
            AgentStatus::Idle,
            IntegrationMode::Compatibility,
        ),
        (
            "codex",
            HarnessKind::CodexCli,
            AgentStatus::NeedsAttention,
            IntegrationMode::Compatibility,
        ),
        (
            "pi",
            HarnessKind::Pi,
            AgentStatus::Idle,
            IntegrationMode::Compatibility,
        ),
        (
            "gemini",
            HarnessKind::GeminiCli,
            AgentStatus::Idle,
            IntegrationMode::Compatibility,
        ),
        (
            "opencode",
            HarnessKind::OpenCode,
            AgentStatus::Error,
            IntegrationMode::Compatibility,
        ),
    ];

    for (session_name, harness, status, mode) in cases {
        let session = summary
            .state
            .inventory
            .sessions
            .iter()
            .find(|session| session.name == session_name)
            .expect("session should exist");
        let pane = session
            .windows
            .first()
            .and_then(|window| window.panes.first())
            .expect("pane should exist");
        let agent = pane.agent.as_ref().expect("agent should exist");

        assert_eq!(
            agent.harness, harness,
            "{session_name} harness should match"
        );
        assert_eq!(agent.status, status, "{session_name} status should match");
        assert_eq!(
            agent.integration_mode, mode,
            "{session_name} integration mode should match"
        );
    }
}
