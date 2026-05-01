mod support;

use clap::Parser;
use foreman::app::{AgentStatus, HarnessKind, IntegrationMode, PaneId};
use foreman::cli::{run, Cli, RunOutcome};
use support::tmux::TmuxFixture;
use tempfile::TempDir;

#[test]
fn codex_runtime_identity_prevents_false_claude_native_warnings() {
    let fixture = TmuxFixture::new();
    let temp_dir = tempfile::tempdir().expect("temp dir should exist");
    let codex_native = temp_dir.path().join("codex-native");
    let claude_native = temp_dir.path().join("claude-native");
    std::fs::create_dir_all(&codex_native).expect("codex native dir should exist");
    std::fs::create_dir_all(&claude_native).expect("claude native dir should exist");

    let codex_one = fixture.new_session(
        "codex-one",
        &codex_shaped_command("Question 1/1 (1 unanswered)\nClaude Code mentioned in history"),
    );
    let codex_two = fixture.new_session(
        "codex-two",
        &codex_shaped_command("Claude Code ready in stale scrollback\nCodex prompt"),
    );
    fixture.wait_for_capture(&codex_one, "Question 1/1");
    fixture.wait_for_capture(&codex_two, "stale scrollback");
    write_signal(
        &codex_native,
        &codex_one,
        r#"{"status":"idle","activity_score":40}"#,
    );
    write_signal(
        &codex_native,
        &codex_two,
        r#"{"status":"idle","activity_score":40}"#,
    );

    let summary =
        bootstrap_with_native_dirs(&fixture, &temp_dir, &claude_native, &codex_native, None);

    for pane_id in [&codex_one, &codex_two] {
        let pane = summary
            .state
            .inventory
            .pane(&PaneId::new(pane_id))
            .expect("pane should exist");
        let agent = pane.agent.as_ref().expect("agent should exist");
        assert_eq!(agent.harness, HarnessKind::CodexCli);
        assert_eq!(agent.integration_mode, IntegrationMode::Native);
    }
    assert_eq!(summary.claude_native.fallback_to_compatibility, 0);
    assert_eq!(summary.codex_native.applied, 2);
}

#[test]
fn mixed_native_fixture_reports_only_real_claude_missing_signals() {
    let fixture = TmuxFixture::new();
    let temp_dir = tempfile::tempdir().expect("temp dir should exist");
    let claude_native = temp_dir.path().join("claude-native");
    let codex_native = temp_dir.path().join("codex-native");
    let pi_native = temp_dir.path().join("pi-native");
    std::fs::create_dir_all(&claude_native).expect("claude native dir should exist");
    std::fs::create_dir_all(&codex_native).expect("codex native dir should exist");
    std::fs::create_dir_all(&pi_native).expect("pi native dir should exist");

    let claude_one = fixture.new_session(
        "claude-one",
        &fixture.shell_command("Claude Code is thinking"),
    );
    let claude_two = fixture.new_session("claude-two", &fixture.shell_command("Claude Code ready"));
    let codex = fixture.new_session(
        "codex",
        &codex_shaped_command("Claude Code stale text\nCodex CLI waiting"),
    );
    let pi = fixture.new_session("pi", &pi_title_command("Claude Code stale text\nPi ready"));

    fixture.wait_for_capture(&claude_one, "Claude Code is thinking");
    fixture.wait_for_capture(&claude_two, "Claude Code ready");
    fixture.wait_for_capture(&codex, "Codex CLI waiting");
    fixture.wait_for_capture(&pi, "Pi ready");

    write_signal(
        &codex_native,
        &codex,
        r#"{"status":"idle","activity_score":40}"#,
    );
    write_signal(
        &codex_native,
        &pi,
        r#"{"status":"idle","activity_score":40}"#,
    );
    write_signal(&pi_native, &pi, r#"{"status":"idle","activity_score":40}"#);

    let summary = bootstrap_with_native_dirs(
        &fixture,
        &temp_dir,
        &claude_native,
        &codex_native,
        Some(&pi_native),
    );

    assert_eq!(summary.claude_native.fallback_to_compatibility, 2);
    assert_eq!(summary.codex_native.applied, 1);
    assert_eq!(summary.pi_native.applied, 1);

    let pi_agent = summary
        .state
        .inventory
        .pane(&PaneId::new(&pi))
        .and_then(|pane| pane.agent.as_ref())
        .expect("pi agent should exist");
    assert_eq!(pi_agent.harness, HarnessKind::Pi);
    assert_eq!(pi_agent.integration_mode, IntegrationMode::Native);
}

#[test]
fn codex_question_ui_does_not_override_working_native_signal() {
    let fixture = TmuxFixture::new();
    let temp_dir = tempfile::tempdir().expect("temp dir should exist");
    let codex_native = temp_dir.path().join("codex-native");
    std::fs::create_dir_all(&codex_native).expect("codex native dir should exist");

    let pane_id = fixture.new_session(
        "codex-question",
        &codex_shaped_command(
            "Question 1/1 (1 unanswered)\nWhat scope do you want this plan to optimize for?\nenter to submit answer",
        ),
    );
    fixture.wait_for_capture(&pane_id, "enter to submit answer");
    write_signal(
        &codex_native,
        &pane_id,
        r#"{"status":"working","activity_score":120}"#,
    );

    let summary = bootstrap_with_native_dirs(
        &fixture,
        &temp_dir,
        &temp_dir.path().join("missing-claude-native"),
        &codex_native,
        None,
    );
    let agent = summary
        .state
        .inventory
        .pane(&PaneId::new(&pane_id))
        .and_then(|pane| pane.agent.as_ref())
        .expect("agent should exist");

    assert_eq!(agent.harness, HarnessKind::CodexCli);
    assert_eq!(agent.integration_mode, IntegrationMode::Native);
    assert_eq!(agent.status, AgentStatus::Working);
}

#[test]
fn codex_working_native_signal_ignores_stale_question_text_in_transcript() {
    let fixture = TmuxFixture::new();
    let temp_dir = tempfile::tempdir().expect("temp dir should exist");
    let codex_native = temp_dir.path().join("codex-native");
    std::fs::create_dir_all(&codex_native).expect("codex native dir should exist");

    let pane_id = fixture.new_session(
        "codex-working",
        &codex_shaped_command(
            "UserPromptSubmit hook failed\n\
             previous turn mentioned false positives waiting for input\n\
             question text from stale transcript\n\
             Run /review on my current changes\n\
             Working (58s - esc to interrupt)",
        ),
    );
    fixture.wait_for_capture(&pane_id, "Working");
    write_signal(
        &codex_native,
        &pane_id,
        r#"{"status":"working","activity_score":120}"#,
    );

    let summary = bootstrap_with_native_dirs(
        &fixture,
        &temp_dir,
        &temp_dir.path().join("missing-claude-native"),
        &codex_native,
        None,
    );
    let agent = summary
        .state
        .inventory
        .pane(&PaneId::new(&pane_id))
        .and_then(|pane| pane.agent.as_ref())
        .expect("agent should exist");

    assert_eq!(agent.harness, HarnessKind::CodexCli);
    assert_eq!(agent.integration_mode, IntegrationMode::Native);
    assert_eq!(agent.status, AgentStatus::Working);
}

#[test]
fn pi_working_native_signal_ignores_confirm_text_in_generated_script() {
    let fixture = TmuxFixture::new();
    let temp_dir = tempfile::tempdir().expect("temp dir should exist");
    let pi_native = temp_dir.path().join("pi-native");
    std::fs::create_dir_all(&pi_native).expect("pi native dir should exist");

    let pane_id = fixture.new_session(
        "pi-script",
        &pi_title_command(
            "write ~/git_repositories/dots/config/tmux/scripts/tmux-safe-restart\n\
             Usage: tmux-safe-restart [--dry-run] [--save-only] [--yes]\n\
             Force a tmux-resurrect save, summarize what will be restored, warn about\n\
             agent panes that need manual resume, and optionally ask for confirmation.\n\
             ... (46 more lines, ctrl+o to expand)\n\
             ⠧ Working...",
        ),
    );
    fixture.wait_for_capture(&pane_id, "Working");
    write_signal(
        &pi_native,
        &pane_id,
        r#"{"status":"working","activity_score":120}"#,
    );

    let summary = bootstrap_with_native_dirs(
        &fixture,
        &temp_dir,
        &temp_dir.path().join("missing-claude-native"),
        &temp_dir.path().join("missing-codex-native"),
        Some(&pi_native),
    );
    let agent = summary
        .state
        .inventory
        .pane(&PaneId::new(&pane_id))
        .and_then(|pane| pane.agent.as_ref())
        .expect("agent should exist");

    assert_eq!(agent.harness, HarnessKind::Pi);
    assert_eq!(agent.integration_mode, IntegrationMode::Native);
    assert_eq!(agent.status, AgentStatus::Working);
}

fn bootstrap_with_native_dirs(
    fixture: &TmuxFixture,
    temp_dir: &TempDir,
    claude_native_dir: &std::path::Path,
    codex_native_dir: &std::path::Path,
    pi_native_dir: Option<&std::path::Path>,
) -> foreman::cli::BootstrapSummary {
    let mut args = vec![
        "foreman".to_string(),
        "--config-file".to_string(),
        temp_dir
            .path()
            .join("config.toml")
            .to_string_lossy()
            .into_owned(),
        "--log-dir".to_string(),
        temp_dir.path().join("logs").to_string_lossy().into_owned(),
        "--tmux-socket".to_string(),
        fixture.socket_path().to_string_lossy().into_owned(),
        "--claude-native-dir".to_string(),
        claude_native_dir.to_string_lossy().into_owned(),
        "--codex-native-dir".to_string(),
        codex_native_dir.to_string_lossy().into_owned(),
    ];
    if let Some(pi_native_dir) = pi_native_dir {
        args.extend([
            "--pi-native-dir".to_string(),
            pi_native_dir.to_string_lossy().into_owned(),
        ]);
    }

    match run(Cli::parse_from(args)).expect("bootstrap should succeed") {
        RunOutcome::Bootstrapped(summary) => *summary,
        other => panic!("expected bootstrapped outcome, got {other:?}"),
    }
}

fn write_signal(native_dir: &std::path::Path, pane_id: &str, contents: &str) {
    std::fs::write(native_dir.join(format!("{pane_id}.json")), contents)
        .expect("native signal should be written");
}

fn codex_shaped_command(banner: &str) -> String {
    python_sleep_command(banner, Some("@openai/codex"), None)
}

fn pi_title_command(banner: &str) -> String {
    python_sleep_command(banner, None, Some("π - fixture"))
}

fn python_sleep_command(banner: &str, arg: Option<&str>, title: Option<&str>) -> String {
    let title_line = title
        .map(|title| {
            format!("import sys; sys.stdout.write('\\x1b]2;{title}\\x07'); sys.stdout.flush();")
        })
        .unwrap_or_default();
    let script =
        format!("import time\n{title_line}\nprint({banner:?}, flush=True)\ntime.sleep(60)\n");
    let arg = arg.map(|arg| format!(" {arg}")).unwrap_or_default();
    let inner = format!("exec python3 -u -c {}{}", shell_escape(&script), arg);
    format!("sh -lc {}", shell_escape(&inner))
}

fn shell_escape(value: &str) -> String {
    format!("'{}'", value.replace('\'', r#"'\''"#))
}
