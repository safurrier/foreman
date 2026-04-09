use crate::app::action::{Action, SelectionDirection};
use crate::app::state::{AppState, Focus, Mode, PaneId, SelectionTarget};
use crate::integrations::stabilize_inventory;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    FocusPane { pane_id: PaneId, close_after: bool },
    Quit,
}

pub fn reduce(state: &mut AppState, action: Action) -> Vec<Effect> {
    match action {
        Action::RequestMode(mode) => {
            if mode.precedence() >= state.mode.precedence() {
                state.mode = mode;
                if mode == Mode::Input {
                    state.focus = Focus::Input;
                }
            }
        }
        Action::CancelMode => {
            state.mode = Mode::Normal;
            state.focus = Focus::Sidebar;
        }
        Action::RequestQuit => {
            return vec![Effect::Quit];
        }
        Action::SetFocus(focus) => {
            state.focus = focus;
        }
        Action::SetSelection(target) => {
            if state.visible_targets().contains(&target) {
                state.selection = Some(target);
                state.focus = Focus::Sidebar;
            }
        }
        Action::MoveSelection(direction) => {
            let visible_targets = state.visible_targets();
            if visible_targets.is_empty() {
                state.selection = None;
                return Vec::new();
            }

            let current_index = state.selection.as_ref().and_then(|target| {
                visible_targets
                    .iter()
                    .position(|candidate| candidate == target)
            });

            let next_index = match (current_index, direction) {
                (Some(index), SelectionDirection::Previous) => index.saturating_sub(1),
                (Some(index), SelectionDirection::Next) => {
                    (index + 1).min(visible_targets.len() - 1)
                }
                (None, SelectionDirection::Previous) => visible_targets.len() - 1,
                (None, SelectionDirection::Next) => 0,
            };

            state.selection = Some(visible_targets[next_index].clone());
        }
        Action::ReplaceInventory(mut inventory) => {
            stabilize_inventory(&state.inventory, &mut inventory);
            state.inventory = inventory;
            state
                .collapsed_sessions
                .retain(|session_id| state.inventory.contains_session(session_id));
            state.reconcile_selection();
        }
        Action::FocusSelectedPane => {
            if let Some(SelectionTarget::Pane(pane_id)) = state.selection.as_ref() {
                return vec![Effect::FocusPane {
                    pane_id: pane_id.clone(),
                    close_after: state.popup_mode,
                }];
            }
        }
        Action::ToggleShowNonAgentSessions => {
            state.filters.show_non_agent_sessions = !state.filters.show_non_agent_sessions;
            state.reconcile_selection();
        }
        Action::ToggleShowNonAgentPanes => {
            state.filters.show_non_agent_panes = !state.filters.show_non_agent_panes;
            state.reconcile_selection();
        }
        Action::ToggleSessionCollapsed(session_id) => {
            if !state.inventory.contains_session(&session_id) {
                return Vec::new();
            }

            if !state.collapsed_sessions.insert(session_id.clone()) {
                state.collapsed_sessions.remove(&session_id);
            }

            state.reconcile_selection();
        }
        Action::SetSortMode(sort_mode) => {
            state.sort_mode = sort_mode;
            state.reconcile_selection();
        }
        Action::SetSearchQuery(search_query) => {
            state.search_query = search_query;
        }
        Action::ClearSearchQuery => {
            state.search_query.clear();
        }
        Action::Noop => {}
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use crate::app::{
        inventory, reduce, Action, AgentStatus, AppState, Focus, HarnessKind, Mode, PaneBuilder,
        SelectionDirection, SelectionTarget, SessionBuilder, SortMode, WindowBuilder,
    };

    fn sample_inventory() -> crate::app::Inventory {
        inventory([
            SessionBuilder::new("alpha").window(
                WindowBuilder::new("alpha:agents").pane(
                    PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                        .status(AgentStatus::Working)
                        .activity_score(10),
                ),
            ),
            SessionBuilder::new("beta").window(
                WindowBuilder::new("beta:agents").pane(
                    PaneBuilder::agent("beta:codex", HarnessKind::CodexCli)
                        .status(AgentStatus::NeedsAttention)
                        .activity_score(4),
                ),
            ),
            SessionBuilder::new("notes")
                .window(WindowBuilder::new("notes:window").pane(PaneBuilder::new("notes:pane"))),
        ])
    }

    #[test]
    fn higher_priority_mode_replaces_lower_priority_mode() {
        let mut state = AppState::default();

        reduce(&mut state, Action::RequestMode(Mode::Input));
        assert_eq!(state.mode, Mode::Input);
        assert_eq!(state.focus, Focus::Input);

        reduce(&mut state, Action::RequestMode(Mode::Help));
        assert_eq!(state.mode, Mode::Help);

        reduce(&mut state, Action::RequestMode(Mode::Search));
        assert_eq!(state.mode, Mode::Help);

        reduce(&mut state, Action::CancelMode);
        assert_eq!(state.mode, Mode::Normal);
    }

    #[test]
    fn refresh_keeps_logical_selection_when_target_still_exists() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("beta:codex".into()));

        let refreshed_inventory = inventory([
            SessionBuilder::new("beta").window(
                WindowBuilder::new("beta:agents").pane(
                    PaneBuilder::agent("beta:codex", HarnessKind::CodexCli)
                        .status(AgentStatus::NeedsAttention)
                        .activity_score(20),
                ),
            ),
            SessionBuilder::new("alpha").window(
                WindowBuilder::new("alpha:agents").pane(
                    PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                        .status(AgentStatus::Working)
                        .activity_score(5),
                ),
            ),
        ]);

        reduce(&mut state, Action::ReplaceInventory(refreshed_inventory));

        assert_eq!(
            state.selection,
            Some(SelectionTarget::Pane("beta:codex".into()))
        );
    }

    #[test]
    fn collapsing_a_session_falls_back_to_the_session_header() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));

        reduce(&mut state, Action::ToggleSessionCollapsed("alpha".into()));

        assert_eq!(
            state.selection,
            Some(SelectionTarget::Session("alpha".into()))
        );
    }

    #[test]
    fn non_agent_visibility_filters_persist_across_refreshes() {
        let mut state = AppState::with_inventory(sample_inventory());

        reduce(&mut state, Action::ToggleShowNonAgentSessions);
        reduce(&mut state, Action::ToggleShowNonAgentPanes);

        let refreshed_inventory = inventory([
            SessionBuilder::new("notes")
                .window(WindowBuilder::new("notes:window").pane(PaneBuilder::new("notes:pane"))),
            SessionBuilder::new("alpha").window(
                WindowBuilder::new("alpha:agents").pane(
                    PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                        .status(AgentStatus::Working)
                        .activity_score(3),
                ),
            ),
        ]);

        reduce(&mut state, Action::ReplaceInventory(refreshed_inventory));

        assert!(state.filters.show_non_agent_sessions);
        assert!(state.filters.show_non_agent_panes);
        assert!(state
            .visible_targets()
            .contains(&SelectionTarget::Pane("notes:pane".into())));
    }

    #[test]
    fn move_selection_uses_sorted_visible_targets() {
        let mut state = AppState::with_inventory(sample_inventory());
        reduce(&mut state, Action::SetSortMode(SortMode::AttentionFirst));
        state.selection = Some(SelectionTarget::Session("beta".into()));

        reduce(&mut state, Action::MoveSelection(SelectionDirection::Next));

        assert_eq!(
            state.selection,
            Some(SelectionTarget::Window("beta:agents".into()))
        );
    }

    #[test]
    fn cancel_mode_resets_to_normal_sidebar() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.mode = Mode::Help;
        state.focus = Focus::Preview;

        reduce(&mut state, Action::CancelMode);

        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.focus, Focus::Sidebar);
    }

    #[test]
    fn focus_selected_pane_emits_tmux_effect_and_respects_popup_mode() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));
        state.popup_mode = true;

        let effects = reduce(&mut state, Action::FocusSelectedPane);

        assert_eq!(
            effects,
            vec![super::Effect::FocusPane {
                pane_id: "alpha:claude".into(),
                close_after: true,
            }]
        );
    }

    #[test]
    fn replace_inventory_debounces_brief_loss_of_working_status() {
        let mut state = AppState::with_inventory(inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                    .status(AgentStatus::Working)
                    .observed_status(AgentStatus::Working)
                    .activity_score(100),
            ),
        )]));
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));

        let refreshed_inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                    .status(AgentStatus::Idle)
                    .observed_status(AgentStatus::Idle)
                    .activity_score(30),
            ),
        )]);

        reduce(&mut state, Action::ReplaceInventory(refreshed_inventory));

        let pane = state
            .inventory
            .pane(&"alpha:claude".into())
            .expect("pane should exist")
            .agent
            .as_ref()
            .expect("agent should exist");
        assert_eq!(pane.status, AgentStatus::Working);
        assert_eq!(pane.observed_status, AgentStatus::Idle);
        assert_eq!(pane.debounce_ticks, 1);
    }
}
