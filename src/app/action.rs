use crate::app::command::Command;
use crate::app::state::{
    AppState, FlashNavigateKind, Focus, Inventory, Mode, PaneId, SelectionTarget, SessionId,
    SortMode, WindowId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionDirection {
    Previous,
    Next,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftEdit {
    InsertChar(char),
    Backspace,
    InsertNewline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    RequestMode(Mode),
    BeginInput,
    BeginSearch,
    BeginFlash {
        kind: FlashNavigateKind,
    },
    CancelMode,
    RequestQuit,
    SetFocus(Focus),
    SetSelection(SelectionTarget),
    MoveSelection(SelectionDirection),
    ReplaceInventory(Inventory),
    FocusSelectedPane,
    CommitSearchSelection,
    OpenRenameModal {
        window_id: WindowId,
        current_name: String,
    },
    OpenSpawnModal {
        session_id: SessionId,
    },
    OpenKillConfirmation {
        pane_id: PaneId,
    },
    EditDraft(DraftEdit),
    SubmitActiveDraft,
    ConfirmKill,
    ToggleShowNonAgentSessions,
    ToggleShowNonAgentPanes,
    ToggleSessionCollapsed(SessionId),
    SetSortMode(SortMode),
    Noop,
}

pub fn action_for_command(state: &AppState, command: Command) -> Action {
    match command {
        Command::NavigateUp => Action::MoveSelection(SelectionDirection::Previous),
        Command::NavigateDown => Action::MoveSelection(SelectionDirection::Next),
        Command::NavigateLeft => Action::SetFocus(state.focus.previous()),
        Command::NavigateRight => Action::SetFocus(state.focus.next()),
        Command::StartInput => {
            if state.selected_pane_id().is_some() {
                Action::BeginInput
            } else {
                Action::Noop
            }
        }
        Command::Select => {
            if state.mode == Mode::Search {
                Action::CommitSearchSelection
            } else {
                match state.selection.as_ref() {
                    Some(SelectionTarget::Session(session_id)) => {
                        Action::ToggleSessionCollapsed(session_id.clone())
                    }
                    Some(SelectionTarget::Pane(_)) => Action::FocusSelectedPane,
                    _ => Action::Noop,
                }
            }
        }
        Command::Cancel => {
            if state.mode != Mode::Normal {
                Action::CancelMode
            } else if state.focus != Focus::Sidebar {
                Action::SetFocus(Focus::Sidebar)
            } else {
                Action::Noop
            }
        }
        Command::FocusSidebar => Action::SetFocus(Focus::Sidebar),
        Command::FocusPreview => Action::SetFocus(Focus::Preview),
        Command::FocusInput => Action::SetFocus(Focus::Input),
        Command::FocusSelectedPane => Action::FocusSelectedPane,
        Command::RequestKill => match state.selected_pane_id() {
            Some(pane_id) => Action::OpenKillConfirmation { pane_id },
            None => Action::Noop,
        },
        Command::InsertChar(ch) => Action::EditDraft(DraftEdit::InsertChar(ch)),
        Command::Backspace => Action::EditDraft(DraftEdit::Backspace),
        Command::InsertNewline => Action::EditDraft(DraftEdit::InsertNewline),
        Command::SubmitDraft => Action::SubmitActiveDraft,
        Command::Confirm => Action::ConfirmKill,
        Command::ToggleHelp => {
            if state.mode == Mode::Help {
                Action::CancelMode
            } else {
                Action::RequestMode(Mode::Help)
            }
        }
        Command::Search => Action::BeginSearch,
        Command::FlashNavigate => Action::BeginFlash {
            kind: FlashNavigateKind::Jump,
        },
        Command::FlashNavigateFocus => Action::BeginFlash {
            kind: FlashNavigateKind::JumpAndFocus,
        },
        Command::RenameWindow => match (state.selected_window_id(), state.selected_window_name()) {
            (Some(window_id), Some(current_name)) => Action::OpenRenameModal {
                window_id,
                current_name,
            },
            _ => Action::Noop,
        },
        Command::SpawnAgent => match state.selected_session_id() {
            Some(session_id) => Action::OpenSpawnModal { session_id },
            None => Action::Noop,
        },
        Command::ToggleNonAgentSessions => Action::ToggleShowNonAgentSessions,
        Command::ToggleNonAgentPanes => Action::ToggleShowNonAgentPanes,
        Command::CycleSortMode => Action::SetSortMode(match state.sort_mode {
            SortMode::RecentActivity => SortMode::AttentionFirst,
            SortMode::AttentionFirst => SortMode::RecentActivity,
        }),
        Command::Quit => Action::RequestQuit,
    }
}

#[cfg(test)]
mod tests {
    use super::{action_for_command, Action, DraftEdit};
    use crate::app::{
        inventory, AppState, Command, FlashNavigateKind, Focus, Mode, PaneBuilder, SelectionTarget,
        SessionBuilder, WindowBuilder,
    };

    fn state_with_selection(selection: Option<SelectionTarget>) -> AppState {
        let inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents")
                .pane(PaneBuilder::new("alpha:helper"))
                .pane(PaneBuilder::new("alpha:main")),
        )]);
        let mut state = AppState::with_inventory(inventory);
        state.selection = selection;
        state
    }

    #[test]
    fn select_toggles_session_headers_and_focuses_panes() {
        let state = state_with_selection(Some(SelectionTarget::Session("alpha".into())));
        assert_eq!(
            action_for_command(&state, Command::Select),
            Action::ToggleSessionCollapsed("alpha".into())
        );

        let state = state_with_selection(Some(SelectionTarget::Pane("alpha:main".into())));
        assert_eq!(
            action_for_command(&state, Command::Select),
            Action::FocusSelectedPane
        );
    }

    #[test]
    fn cancel_prefers_mode_exit_then_focus_reset() {
        let mut state = state_with_selection(None);
        state.mode = Mode::Help;
        state.focus = Focus::Preview;
        assert_eq!(
            action_for_command(&state, Command::Cancel),
            Action::CancelMode
        );

        state.mode = Mode::Normal;
        assert_eq!(
            action_for_command(&state, Command::Cancel),
            Action::SetFocus(Focus::Sidebar)
        );
    }

    #[test]
    fn navigate_left_and_right_cycle_focus() {
        let mut state = state_with_selection(None);
        state.focus = Focus::Sidebar;
        assert_eq!(
            action_for_command(&state, Command::NavigateRight),
            Action::SetFocus(Focus::Preview)
        );

        state.focus = Focus::Input;
        assert_eq!(
            action_for_command(&state, Command::NavigateLeft),
            Action::SetFocus(Focus::Preview)
        );
    }

    #[test]
    fn editing_commands_route_to_draft_actions() {
        let state = state_with_selection(Some(SelectionTarget::Pane("alpha:main".into())));

        assert_eq!(
            action_for_command(&state, Command::StartInput),
            Action::BeginInput
        );
        assert_eq!(
            action_for_command(&state, Command::InsertChar('x')),
            Action::EditDraft(DraftEdit::InsertChar('x'))
        );
        assert_eq!(
            action_for_command(&state, Command::SubmitDraft),
            Action::SubmitActiveDraft
        );
    }

    #[test]
    fn rename_spawn_and_kill_actions_capture_selected_targets() {
        let state = state_with_selection(Some(SelectionTarget::Pane("alpha:main".into())));

        assert_eq!(
            action_for_command(&state, Command::RenameWindow),
            Action::OpenRenameModal {
                window_id: "alpha:agents".into(),
                current_name: "alpha:agents".to_string(),
            }
        );
        assert_eq!(
            action_for_command(&state, Command::SpawnAgent),
            Action::OpenSpawnModal {
                session_id: "alpha".into(),
            }
        );
        assert_eq!(
            action_for_command(&state, Command::RequestKill),
            Action::OpenKillConfirmation {
                pane_id: "alpha:main".into(),
            }
        );
    }

    #[test]
    fn search_and_flash_commands_open_navigation_modes() {
        let state = state_with_selection(Some(SelectionTarget::Pane("alpha:main".into())));

        assert_eq!(
            action_for_command(&state, Command::Search),
            Action::BeginSearch
        );
        assert_eq!(
            action_for_command(&state, Command::FlashNavigate),
            Action::BeginFlash {
                kind: FlashNavigateKind::Jump,
            }
        );
        assert_eq!(
            action_for_command(&state, Command::FlashNavigateFocus),
            Action::BeginFlash {
                kind: FlashNavigateKind::JumpAndFocus,
            }
        );
    }

    #[test]
    fn select_commits_search_mode() {
        let mut state = state_with_selection(Some(SelectionTarget::Pane("alpha:main".into())));
        state.mode = Mode::Search;

        assert_eq!(
            action_for_command(&state, Command::Select),
            Action::CommitSearchSelection
        );
    }
}
