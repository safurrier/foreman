mod support;

use foreman::adapters::tmux::{SystemTmuxBackend, TmuxAdapter};
use foreman::app::{
    reduce, Action, AppState, DraftEdit, Effect, FlashNavigateKind, SelectionTarget,
};
use support::tmux::TmuxFixture;

#[test]
fn flash_jump_and_focus_uses_live_tmux_target() {
    let fixture = TmuxFixture::new();
    let primary = fixture.new_session("alpha", &fixture.shell_command("Claude Code ready"));
    let helper = fixture.split_window(
        &primary,
        &fixture.shell_command("Codex CLI waiting for your input"),
    );
    fixture.wait_for_capture(&helper, "Codex CLI waiting");

    let adapter = TmuxAdapter::new(SystemTmuxBackend::new(Some(
        fixture.socket_path().to_path_buf(),
    )));
    let inventory = adapter.load_inventory(20).expect("inventory should load");
    let mut state = AppState::with_inventory(inventory);

    reduce(
        &mut state,
        Action::BeginFlash {
            kind: FlashNavigateKind::JumpAndFocus,
        },
    );

    let label = state
        .flash_label_for_target(&SelectionTarget::Pane(helper.clone().into()))
        .expect("flash label should exist");

    let mut effects = Vec::new();
    for ch in label.chars() {
        effects = reduce(&mut state, Action::EditDraft(DraftEdit::InsertChar(ch)));
    }

    assert_eq!(
        effects,
        vec![Effect::FocusPane {
            pane_id: helper.clone().into(),
            close_after: false,
        }]
    );

    adapter
        .focus_pane(&helper.clone().into())
        .expect("focus should succeed");
    assert_eq!(fixture.active_pane_in("alpha"), helper);
}
