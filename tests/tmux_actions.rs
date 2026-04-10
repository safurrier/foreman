mod support;

use foreman::adapters::tmux::{SystemTmuxBackend, TmuxAdapter};
use support::tmux::TmuxFixture;

#[test]
fn system_tmux_backend_sends_multiline_input_to_target_pane() {
    let fixture = TmuxFixture::new();
    let pane_id = fixture.new_session(
        "alpha",
        r#"sh -lc 'while IFS= read -r line; do printf "%s\n" "INPUT:$line"; done'"#,
    );

    let adapter = TmuxAdapter::new(SystemTmuxBackend::new(Some(
        fixture.socket_path().to_path_buf(),
    )));
    let result = adapter
        .send_input(&pane_id.clone().into(), "hello\nsecond line")
        .expect("send input should succeed");

    assert_eq!(result.bytes_sent, "hello\nsecond line".len());
    fixture.wait_for_capture(&pane_id, "INPUT:hello");
    fixture.wait_for_capture(&pane_id, "INPUT:second line");
}

#[test]
fn system_tmux_backend_renames_and_spawns_windows() {
    let fixture = TmuxFixture::new();
    let pane_id = fixture.new_session("alpha", &fixture.shell_command("Claude Code ready"));
    fixture.wait_for_capture(&pane_id, "Claude Code ready");

    let adapter = TmuxAdapter::new(SystemTmuxBackend::new(Some(
        fixture.socket_path().to_path_buf(),
    )));
    let initial_inventory = adapter.load_inventory(20).expect("inventory should load");
    let session = initial_inventory
        .sessions
        .first()
        .expect("session should exist");
    let session_id = session.id.clone();
    let window_id = session
        .windows
        .first()
        .expect("window should exist")
        .id
        .clone();

    adapter
        .rename_window(&window_id, "renamed")
        .expect("rename should succeed");
    let spawned = adapter
        .spawn_window(
            &session_id,
            &fixture.shell_command("Codex CLI waiting for your input"),
        )
        .expect("spawn should succeed");
    fixture.wait_for_capture(spawned.pane_id.as_str(), "Codex CLI waiting");

    let refreshed_inventory = adapter.load_inventory(20).expect("inventory should reload");
    let refreshed_session = refreshed_inventory
        .session(&session_id)
        .expect("session should still exist");

    assert_eq!(refreshed_session.windows.len(), 2);
    assert!(refreshed_session
        .windows
        .iter()
        .any(|window| window.name == "renamed"));
    assert!(refreshed_session
        .windows
        .iter()
        .any(|window| window.id == spawned.window_id));
}

#[test]
fn system_tmux_backend_kills_target_pane() {
    let fixture = TmuxFixture::new();
    let pane_id = fixture.new_session("alpha", &fixture.shell_command("plain shell"));
    fixture.wait_for_capture(&pane_id, "plain shell");
    let keep_alive = fixture.new_session("beta", &fixture.shell_command("keep alive"));
    fixture.wait_for_capture(&keep_alive, "keep alive");

    let adapter = TmuxAdapter::new(SystemTmuxBackend::new(Some(
        fixture.socket_path().to_path_buf(),
    )));
    adapter
        .kill_pane(&pane_id.clone().into())
        .expect("kill should succeed");

    let refreshed_inventory = adapter.load_inventory(20).expect("inventory should reload");
    assert!(refreshed_inventory.pane(&pane_id.clone().into()).is_none());
    assert!(refreshed_inventory.pane(&keep_alive.into()).is_some());
}
