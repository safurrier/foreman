mod support;

use foreman::adapters::tmux::{SystemTmuxBackend, TmuxAdapter};
use foreman::app::{reduce, Action, AppState, Effect, SelectionTarget};
use support::tmux::TmuxFixture;

#[test]
fn system_tmux_backend_focuses_target_pane() {
    let fixture = TmuxFixture::new();
    let _alpha_main = fixture.new_session("alpha", &fixture.shell_command("Claude Code ready"));
    let alpha_helper = fixture.split_window("alpha:1", &fixture.shell_command("plain shell"));
    fixture.wait_for_capture(&alpha_helper, "plain shell");

    let adapter = TmuxAdapter::new(SystemTmuxBackend::new(Some(
        fixture.socket_path().to_path_buf(),
    )));
    adapter
        .focus_pane(&alpha_helper.clone().into())
        .expect("focus should succeed");

    assert_eq!(fixture.active_pane_in("alpha:1"), alpha_helper);
}

#[test]
fn popup_focus_effect_requests_close_after_success() {
    let mut state = AppState {
        selection: Some(SelectionTarget::Pane("%42".into())),
        popup_mode: true,
        ..AppState::default()
    };

    let effects = reduce(&mut state, Action::FocusSelectedPane);

    assert_eq!(
        effects,
        vec![Effect::FocusPane {
            pane_id: "%42".into(),
            close_after: true,
        }]
    );
}
