mod support;

use foreman::adapters::tmux::{SystemTmuxBackend, TmuxAdapter};
use std::thread;
use std::time::Duration;
use support::tmux::TmuxFixture;
use tempfile::tempdir;

fn foreman_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman")
}

#[test]
fn interactive_binary_renders_dashboard_and_sends_input_to_selected_agent() {
    let fixture = TmuxFixture::new();
    let agent_pane = fixture.new_session(
        "alpha",
        r#"sh -lc 'printf "%s\n" "Claude Code ready"; while IFS= read -r line; do printf "%s\n" "INPUT:$line"; done'"#,
    );
    fixture.wait_for_capture(&agent_pane, "Claude Code ready");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let dashboard_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --no-notify",
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman");
    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");

    fixture.send_keys(&dashboard_pane, &["j", "j"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "Press i to compose for");

    fixture.send_keys(&dashboard_pane, &["i", "h", "i", "C-s"]);
    fixture.wait_for_capture(&agent_pane, "INPUT:hi");

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
}

#[test]
fn interactive_binary_popup_focus_action_exits_after_success() {
    let fixture = TmuxFixture::new();
    let helper_pane = fixture.new_session("alpha", &fixture.shell_command("plain shell"));
    let agent_pane =
        fixture.split_window(&helper_pane, &fixture.shell_command("Claude Code ready"));
    fixture.wait_for_capture(&agent_pane, "Claude Code ready");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let dashboard_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --no-notify --popup",
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman");
    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");

    fixture.send_keys(&dashboard_pane, &["j", "j", "f"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
    assert_eq!(fixture.active_pane_in("alpha"), agent_pane);
}

#[test]
fn interactive_binary_spawn_modal_submits_with_enter() {
    let fixture = TmuxFixture::new();
    let agent_pane = fixture.new_session("alpha", &fixture.shell_command("Claude Code ready"));
    fixture.wait_for_capture(&agent_pane, "Claude Code ready");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let dashboard_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --no-notify",
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");
    fixture.send_keys(
        &dashboard_pane,
        &["N", "s", "l", "e", "e", "p", "Space", "6", "0", "Enter"],
    );

    let adapter = TmuxAdapter::new(SystemTmuxBackend::new(Some(
        fixture.socket_path().to_path_buf(),
    )));
    let mut window_count = 0;
    for _ in 0..20 {
        let inventory = adapter.load_inventory(20).expect("inventory should load");
        let session = inventory
            .session(&"alpha".into())
            .or_else(|| {
                inventory
                    .sessions
                    .iter()
                    .find(|session| session.name == "alpha")
            })
            .expect("alpha session should exist");
        window_count = session.windows.len();
        if window_count == 2 {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(window_count, 2);

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
}
