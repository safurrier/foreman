mod support;

use clap::Parser;
use foreman::app::{AgentStatus, HarnessKind, IntegrationMode, Pane, PaneId};
use foreman::cli::{run, Cli, RunOutcome};
use support::tmux::TmuxFixture;
use tempfile::tempdir;

fn claude_hook_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman-claude-hook")
}

fn codex_hook_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman-codex-hook")
}

fn pi_hook_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman-pi-hook")
}

#[test]
fn claude_tmux_hook_signal_promotes_foreman_native_overlay() {
    let fixture = TmuxFixture::new();
    let temp_dir = tempdir().expect("temp dir should exist");
    let native_dir = temp_dir.path().join("claude-native");
    let command = stdin_hook_command(
        "Claude Code is thinking about hooks",
        claude_hook_bin(),
        &native_dir,
        r#"{"hook_event_name":"UserPromptSubmit","prompt":"smoke"}"#,
    );

    let pane_id = fixture.new_session("claude", &command);
    fixture.wait_for_capture(&pane_id, "Claude Code is thinking");
    wait_for_signal(&native_dir, &pane_id);

    let summary = bootstrap_with_native_dirs(&fixture, &temp_dir, Some(&native_dir), None, None);
    let pane = pane_by_id(&summary.state.inventory, &pane_id);
    let agent = pane
        .agent
        .as_ref()
        .expect("pane should be recognized as agent");

    assert_eq!(agent.harness, HarnessKind::ClaudeCode);
    assert_eq!(agent.integration_mode, IntegrationMode::Native);
    assert_eq!(agent.status, AgentStatus::Working);
    assert_eq!(summary.claude_native.applied, 1);
    assert_eq!(summary.claude_native.fallback_to_compatibility, 0);
}

#[test]
fn codex_tmux_hook_signal_promotes_foreman_native_overlay() {
    let fixture = TmuxFixture::new();
    let temp_dir = tempdir().expect("temp dir should exist");
    let native_dir = temp_dir.path().join("codex-native");
    let command = stdin_hook_command(
        "Codex is planning a hook fix",
        codex_hook_bin(),
        &native_dir,
        r#"{"hook_event_name":"UserPromptSubmit","prompt":"smoke"}"#,
    );

    let pane_id = fixture.new_session("codex", &command);
    fixture.wait_for_capture(&pane_id, "Codex is planning");
    wait_for_signal(&native_dir, &pane_id);

    let summary = bootstrap_with_native_dirs(&fixture, &temp_dir, None, Some(&native_dir), None);
    let pane = pane_by_id(&summary.state.inventory, &pane_id);
    let agent = pane
        .agent
        .as_ref()
        .expect("pane should be recognized as agent");

    assert_eq!(agent.harness, HarnessKind::CodexCli);
    assert_eq!(agent.integration_mode, IntegrationMode::Native);
    assert_eq!(agent.status, AgentStatus::Working);
    assert_eq!(summary.codex_native.applied, 1);
    assert_eq!(summary.codex_native.fallback_to_compatibility, 0);
}

#[test]
fn pi_tmux_hook_signal_promotes_foreman_native_overlay() {
    let fixture = TmuxFixture::new();
    let temp_dir = tempdir().expect("temp dir should exist");
    let native_dir = temp_dir.path().join("pi-native");
    let command = pi_hook_command(
        "Pi is thinking about hooks",
        pi_hook_bin(),
        &native_dir,
        "agent-start",
    );

    let pane_id = fixture.new_session("pi", &command);
    fixture.wait_for_capture(&pane_id, "Pi is thinking");
    wait_for_signal(&native_dir, &pane_id);

    let summary = bootstrap_with_native_dirs(&fixture, &temp_dir, None, None, Some(&native_dir));
    let pane = pane_by_id(&summary.state.inventory, &pane_id);
    let agent = pane
        .agent
        .as_ref()
        .expect("pane should be recognized as agent");

    assert_eq!(agent.harness, HarnessKind::Pi);
    assert_eq!(agent.integration_mode, IntegrationMode::Native);
    assert_eq!(agent.status, AgentStatus::Working);
    assert_eq!(summary.pi_native.applied, 1);
    assert_eq!(summary.pi_native.fallback_to_compatibility, 0);
}

fn stdin_hook_command(
    banner: &str,
    hook_bin: &str,
    native_dir: &std::path::Path,
    payload: &str,
) -> String {
    format!(
        "sh -lc {}",
        shell_escape(format!(
            "printf '%s\\n' {}; printf '%s' {} | {} --native-dir {}; exec sleep 60",
            shell_escape(banner),
            shell_escape(payload),
            shell_escape(hook_bin),
            shell_escape(native_dir.display().to_string())
        ))
    )
}

fn pi_hook_command(
    banner: &str,
    hook_bin: &str,
    native_dir: &std::path::Path,
    event: &str,
) -> String {
    format!(
        "sh -lc {}",
        shell_escape(format!(
            "printf '%s\\n' {}; {} --native-dir {} --event {}; exec sleep 60",
            shell_escape(banner),
            shell_escape(hook_bin),
            shell_escape(native_dir.display().to_string()),
            shell_escape(event)
        ))
    )
}

fn bootstrap_with_native_dirs(
    fixture: &TmuxFixture,
    temp_dir: &tempfile::TempDir,
    claude_native_dir: Option<&std::path::Path>,
    codex_native_dir: Option<&std::path::Path>,
    pi_native_dir: Option<&std::path::Path>,
) -> foreman::cli::BootstrapSummary {
    let config_file = temp_dir.path().join("config.toml");
    let log_dir = temp_dir.path().join("logs");
    let mut args = vec![
        "foreman".to_string(),
        "--config-file".to_string(),
        config_file.to_string_lossy().into_owned(),
        "--log-dir".to_string(),
        log_dir.to_string_lossy().into_owned(),
        "--tmux-socket".to_string(),
        fixture.socket_path().to_string_lossy().into_owned(),
    ];

    if let Some(native_dir) = claude_native_dir {
        args.extend([
            "--claude-native-dir".to_string(),
            native_dir.to_string_lossy().into_owned(),
        ]);
    }
    if let Some(native_dir) = codex_native_dir {
        args.extend([
            "--codex-native-dir".to_string(),
            native_dir.to_string_lossy().into_owned(),
        ]);
    }
    if let Some(native_dir) = pi_native_dir {
        args.extend([
            "--pi-native-dir".to_string(),
            native_dir.to_string_lossy().into_owned(),
        ]);
    }

    let cli = Cli::parse_from(args);
    match run(cli).expect("bootstrap should succeed") {
        RunOutcome::Bootstrapped(summary) => *summary,
        other => panic!("expected bootstrapped outcome, got {other:?}"),
    }
}

fn pane_by_id<'a>(inventory: &'a foreman::app::Inventory, pane_id: &str) -> &'a Pane {
    inventory
        .pane(&PaneId::new(pane_id))
        .expect("pane should exist in inventory")
}

fn wait_for_signal(native_dir: &std::path::Path, pane_id: &str) {
    let signal_path = native_dir.join(format!("{pane_id}.json"));
    for _ in 0..40 {
        if signal_path.exists() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    panic!("native signal was not written: {}", signal_path.display());
}

fn shell_escape(value: impl AsRef<str>) -> String {
    format!("'{}'", value.as_ref().replace('\'', r#"'\''"#))
}
