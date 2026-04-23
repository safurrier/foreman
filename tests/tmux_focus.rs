mod support;

use foreman::adapters::tmux::{SystemTmuxBackend, TmuxAdapter};
use foreman::app::{
    inventory, reduce, Action, AppState, Effect, HarnessKind, PaneBuilder, SelectionTarget,
    SessionBuilder, WindowBuilder,
};
use support::tmux::TmuxFixture;

#[test]
fn system_tmux_backend_focuses_target_pane() {
    let fixture = TmuxFixture::new();
    let alpha_main = fixture.new_session("alpha", &fixture.shell_command("Claude Code ready"));
    let alpha_helper = fixture.split_window(&alpha_main, &fixture.shell_command("plain shell"));
    fixture.wait_for_capture(&alpha_helper, "plain shell");

    let adapter = TmuxAdapter::new(SystemTmuxBackend::new(Some(
        fixture.socket_path().to_path_buf(),
    )));
    adapter
        .focus_pane(&alpha_helper.clone().into())
        .expect("focus should succeed");

    assert_eq!(fixture.active_pane_in("alpha"), alpha_helper);
}

#[test]
fn popup_focus_effect_requests_close_after_success() {
    let inventory =
        inventory([SessionBuilder::new("alpha")
            .window(WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("%42", HarnessKind::ClaudeCode).working_dir("/tmp/alpha"),
            ))]);
    let mut state = AppState::with_inventory(inventory);
    state.selection = Some(SelectionTarget::Pane("%42".into()));
    state.popup_mode = true;

    let effects = reduce(&mut state, Action::FocusSelectedPane);

    assert_eq!(
        effects,
        vec![Effect::FocusPane {
            pane_id: "%42".into(),
            close_after: true,
        }]
    );
}
