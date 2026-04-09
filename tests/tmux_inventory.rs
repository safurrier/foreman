mod support;

use clap::Parser;
use foreman::app::{reduce, Action, SelectionTarget};
use foreman::cli::{run, Cli, RunOutcome};
use support::tmux::TmuxFixture;
use tempfile::tempdir;

fn foreman_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman")
}

fn bootstrap_fixture() -> (TmuxFixture, String, String, String) {
    let fixture = TmuxFixture::new();
    let alpha_pane = fixture.new_session("alpha", &fixture.shell_command("Claude Code ready"));
    let notes_helper = fixture.split_window("alpha:1", &fixture.shell_command("plain shell"));
    let beta_pane = fixture.new_session("beta", &fixture.shell_command("Codex CLI waiting"));
    let notes_pane = fixture.new_session("notes", &fixture.shell_command("plain shell"));

    fixture.wait_for_capture(&alpha_pane, "Claude Code ready");
    fixture.wait_for_capture(&beta_pane, "Codex CLI waiting");
    fixture.wait_for_capture(&notes_helper, "plain shell");
    fixture.wait_for_capture(&notes_pane, "plain shell");

    (fixture, alpha_pane, notes_helper, notes_pane)
}

#[test]
fn bootstrap_with_real_tmux_loads_multi_session_inventory_and_visibility_toggles() {
    let (fixture, _alpha_pane, notes_helper, notes_pane) = bootstrap_fixture();
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

    assert_eq!(summary.inventory.total_sessions, 3);
    assert_eq!(summary.inventory.total_panes, 4);
    assert_eq!(summary.inventory.visible_sessions, 2);
    assert_eq!(summary.inventory.visible_panes, 2);

    let selected = summary
        .state
        .selection
        .as_ref()
        .expect("selection should exist");
    match selected {
        SelectionTarget::Session(session_id) => {
            assert_eq!(
                summary
                    .state
                    .inventory
                    .session(session_id)
                    .expect("session should exist")
                    .name,
                "alpha"
            );
        }
        other => panic!("expected session selection, got {other:?}"),
    }

    let mut state = summary.state.clone();
    reduce(&mut state, Action::ToggleShowNonAgentPanes);
    assert_eq!(state.inventory_summary().visible_panes, 3);
    assert!(state
        .visible_targets()
        .contains(&SelectionTarget::Pane(notes_helper.into())));

    reduce(&mut state, Action::ToggleShowNonAgentSessions);
    assert_eq!(state.inventory_summary().visible_sessions, 3);
    assert_eq!(state.inventory_summary().visible_panes, 4);
    assert!(state
        .visible_targets()
        .contains(&SelectionTarget::Pane(notes_pane.into())));
}

#[test]
fn binary_bootstrap_logs_real_tmux_inventory_summary() {
    let (fixture, _alpha_pane, _notes_helper, _notes_pane) = bootstrap_fixture();
    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");

    let output = std::process::Command::new(foreman_bin())
        .arg("--bootstrap-only")
        .args(["--config-file"])
        .arg(config_dir.path().join("config.toml"))
        .args(["--log-dir"])
        .arg(log_dir.path())
        .args(["--tmux-socket"])
        .arg(fixture.socket_path())
        .output()
        .expect("command should run");

    assert!(output.status.success());

    let log_contents =
        std::fs::read_to_string(log_dir.path().join("latest.log")).expect("log should exist");
    assert!(log_contents.contains("inventory_loaded"));
    assert!(log_contents.contains("sessions=3"));
    assert!(log_contents.contains("panes=4"));
    assert!(log_contents.contains("visible_sessions=2"));
    assert!(log_contents.contains("visible_panes=2"));
}
