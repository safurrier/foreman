use crate::app::command::Command;
use crate::app::state::{AppState, Focus, Inventory, Mode, SelectionTarget, SessionId, SortMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionDirection {
    Previous,
    Next,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    RequestMode(Mode),
    CancelMode,
    RequestQuit,
    SetFocus(Focus),
    SetSelection(SelectionTarget),
    MoveSelection(SelectionDirection),
    ReplaceInventory(Inventory),
    FocusSelectedPane,
    ToggleShowNonAgentSessions,
    ToggleShowNonAgentPanes,
    ToggleSessionCollapsed(SessionId),
    SetSortMode(SortMode),
    SetSearchQuery(String),
    ClearSearchQuery,
    Noop,
}

pub fn action_for_command(state: &AppState, command: Command) -> Action {
    match command {
        Command::NavigateUp => Action::MoveSelection(SelectionDirection::Previous),
        Command::NavigateDown => Action::MoveSelection(SelectionDirection::Next),
        Command::NavigateLeft => Action::SetFocus(state.focus.previous()),
        Command::NavigateRight => Action::SetFocus(state.focus.next()),
        Command::Select => match state.selection.as_ref() {
            Some(SelectionTarget::Session(session_id)) => {
                Action::ToggleSessionCollapsed(session_id.clone())
            }
            Some(SelectionTarget::Pane(_)) => Action::FocusSelectedPane,
            _ => Action::Noop,
        },
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
        Command::ToggleHelp => {
            if state.mode == Mode::Help {
                Action::CancelMode
            } else {
                Action::RequestMode(Mode::Help)
            }
        }
        Command::Search => Action::RequestMode(Mode::Search),
        Command::FlashNavigate => Action::RequestMode(Mode::FlashNavigate),
        Command::RenameWindow => Action::RequestMode(Mode::Rename),
        Command::SpawnAgent => Action::RequestMode(Mode::Spawn),
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
    use super::{action_for_command, Action};
    use crate::app::{
        inventory, AppState, Command, Focus, Mode, PaneBuilder, SelectionTarget, SessionBuilder,
        WindowBuilder,
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
}
