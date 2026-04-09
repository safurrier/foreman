use crate::app::action::{Action, DraftEdit, SelectionDirection};
use crate::app::state::{
    AppState, Focus, ModalState, Mode, PaneId, SelectionTarget, SessionId, WindowId,
};
use crate::integrations::stabilize_inventory;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    FocusPane {
        pane_id: PaneId,
        close_after: bool,
    },
    SendInput {
        pane_id: PaneId,
        text: String,
    },
    RenameWindow {
        window_id: WindowId,
        name: String,
    },
    SpawnWindow {
        session_id: SessionId,
        command: String,
    },
    KillPane {
        pane_id: PaneId,
    },
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
        Action::BeginInput => {
            if state.selected_pane_id().is_some() {
                state.mode = Mode::Input;
                state.focus = Focus::Input;
            }
        }
        Action::CancelMode => {
            if state.mode == Mode::Input {
                state.input_draft.clear();
            }
            state.mode = Mode::Normal;
            state.focus = Focus::Sidebar;
            state.modal = None;
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
            reconcile_interaction_state(state);
        }
        Action::FocusSelectedPane => {
            if let Some(SelectionTarget::Pane(pane_id)) = state.selection.as_ref() {
                return vec![Effect::FocusPane {
                    pane_id: pane_id.clone(),
                    close_after: state.popup_mode,
                }];
            }
        }
        Action::OpenRenameModal {
            window_id,
            current_name,
        } => {
            if state.inventory.window(&window_id).is_some() {
                state.mode = Mode::Rename;
                state.modal = Some(ModalState::rename_window(window_id, current_name));
            }
        }
        Action::OpenSpawnModal { session_id } => {
            if state.inventory.contains_session(&session_id) {
                state.mode = Mode::Spawn;
                state.modal = Some(ModalState::spawn_window(session_id));
            }
        }
        Action::OpenKillConfirmation { pane_id } => {
            if state.inventory.pane(&pane_id).is_some() {
                state.mode = Mode::ConfirmKill;
                state.modal = Some(ModalState::confirm_kill(pane_id));
            }
        }
        Action::EditDraft(edit) => {
            if state.mode == Mode::Input {
                apply_draft_edit(&mut state.input_draft, edit);
            } else if let Some(draft) = state.modal.as_mut().and_then(ModalState::draft_mut) {
                apply_draft_edit(draft, edit);
            }
        }
        Action::SubmitActiveDraft => match state.mode {
            Mode::Input => {
                if let Some(pane_id) = state.selected_pane_id() {
                    if state.input_draft.is_blank() {
                        return Vec::new();
                    }

                    let text = state.input_draft.take();
                    state.mode = Mode::Normal;
                    return vec![Effect::SendInput { pane_id, text }];
                }
            }
            Mode::Rename => {
                if let Some(ModalState::RenameWindow { window_id, draft }) = state.modal.take() {
                    if draft.is_blank() {
                        state.modal = Some(ModalState::rename_window(window_id, draft.text));
                        return Vec::new();
                    }

                    state.mode = Mode::Normal;
                    return vec![Effect::RenameWindow {
                        window_id,
                        name: draft.text,
                    }];
                }
            }
            Mode::Spawn => {
                if let Some(ModalState::SpawnWindow { session_id, draft }) = state.modal.take() {
                    if draft.is_blank() {
                        state.modal = Some(ModalState::SpawnWindow { session_id, draft });
                        return Vec::new();
                    }

                    state.mode = Mode::Normal;
                    return vec![Effect::SpawnWindow {
                        session_id,
                        command: draft.text,
                    }];
                }
            }
            Mode::Normal
            | Mode::PreviewScroll
            | Mode::Search
            | Mode::FlashNavigate
            | Mode::Help
            | Mode::ConfirmKill => {}
        },
        Action::ConfirmKill => {
            if let Some(ModalState::ConfirmKill { pane_id }) = state.modal.take() {
                state.mode = Mode::Normal;
                return vec![Effect::KillPane { pane_id }];
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

fn apply_draft_edit(draft: &mut crate::app::TextDraft, edit: DraftEdit) {
    match edit {
        DraftEdit::InsertChar(ch) => draft.push_char(ch),
        DraftEdit::Backspace => draft.backspace(),
        DraftEdit::InsertNewline => draft.insert_newline(),
    }
}

fn reconcile_interaction_state(state: &mut AppState) {
    if state.mode == Mode::Input && state.selected_pane_id().is_none() {
        state.mode = Mode::Normal;
        state.focus = Focus::Sidebar;
        state.input_draft.clear();
    }

    let modal_is_valid = match state.modal.as_ref() {
        Some(ModalState::RenameWindow { window_id, .. }) => {
            state.inventory.window(window_id).is_some()
        }
        Some(ModalState::SpawnWindow { session_id, .. }) => {
            state.inventory.contains_session(session_id)
        }
        Some(ModalState::ConfirmKill { pane_id }) => state.inventory.pane(pane_id).is_some(),
        None => true,
    };

    if !modal_is_valid {
        state.mode = Mode::Normal;
        state.modal = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::app::{
        inventory, reduce, Action, AgentStatus, AppState, DraftEdit, Focus, HarnessKind,
        ModalState, Mode, PaneBuilder, SelectionDirection, SelectionTarget, SessionBuilder,
        SortMode, WindowBuilder,
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

    #[test]
    fn input_mode_builds_multiline_draft_and_emits_send_effect() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));

        reduce(&mut state, Action::BeginInput);
        reduce(&mut state, Action::EditDraft(DraftEdit::InsertChar('h')));
        reduce(&mut state, Action::EditDraft(DraftEdit::InsertChar('i')));
        reduce(&mut state, Action::EditDraft(DraftEdit::InsertNewline));
        reduce(
            &mut state,
            Action::EditDraft(DraftEdit::InsertChar('\u{00E9}')),
        );

        let effects = reduce(&mut state, Action::SubmitActiveDraft);

        assert_eq!(
            effects,
            vec![super::Effect::SendInput {
                pane_id: "alpha:claude".into(),
                text: "hi\n\u{00E9}".to_string(),
            }]
        );
        assert_eq!(state.mode, Mode::Normal);
        assert!(state.input_draft.text.is_empty());
    }

    #[test]
    fn backspace_respects_utf8_boundaries() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));
        reduce(&mut state, Action::BeginInput);
        reduce(
            &mut state,
            Action::EditDraft(DraftEdit::InsertChar('\u{1F916}')),
        );

        reduce(&mut state, Action::EditDraft(DraftEdit::Backspace));

        assert!(state.input_draft.text.is_empty());
    }

    #[test]
    fn rename_mode_prefills_current_window_and_submits_effect() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));

        reduce(
            &mut state,
            Action::OpenRenameModal {
                window_id: "alpha:agents".into(),
                current_name: "alpha:agents".to_string(),
            },
        );

        assert_eq!(
            state.modal,
            Some(ModalState::rename_window(
                "alpha:agents".into(),
                "alpha:agents"
            ))
        );

        reduce(&mut state, Action::EditDraft(DraftEdit::Backspace));
        reduce(&mut state, Action::EditDraft(DraftEdit::InsertChar('x')));
        let effects = reduce(&mut state, Action::SubmitActiveDraft);

        assert_eq!(
            effects,
            vec![super::Effect::RenameWindow {
                window_id: "alpha:agents".into(),
                name: "alpha:agentx".to_string(),
            }]
        );
        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.modal, None);
    }

    #[test]
    fn spawn_mode_targets_selected_session_and_emits_command() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));

        reduce(
            &mut state,
            Action::OpenSpawnModal {
                session_id: "alpha".into(),
            },
        );
        for ch in "claude".chars() {
            reduce(&mut state, Action::EditDraft(DraftEdit::InsertChar(ch)));
        }

        let effects = reduce(&mut state, Action::SubmitActiveDraft);

        assert_eq!(
            effects,
            vec![super::Effect::SpawnWindow {
                session_id: "alpha".into(),
                command: "claude".to_string(),
            }]
        );
        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.modal, None);
    }

    #[test]
    fn kill_requires_confirmation_before_emitting_effect() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));

        let first_effects = reduce(
            &mut state,
            Action::OpenKillConfirmation {
                pane_id: "alpha:claude".into(),
            },
        );
        assert!(first_effects.is_empty());
        assert_eq!(state.mode, Mode::ConfirmKill);
        assert_eq!(
            state.modal,
            Some(ModalState::confirm_kill("alpha:claude".into()))
        );

        let second_effects = reduce(&mut state, Action::ConfirmKill);
        assert_eq!(
            second_effects,
            vec![super::Effect::KillPane {
                pane_id: "alpha:claude".into(),
            }]
        );
        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.modal, None);
    }

    #[test]
    fn replace_inventory_clears_invalid_modal_targets() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));
        reduce(
            &mut state,
            Action::OpenKillConfirmation {
                pane_id: "alpha:claude".into(),
            },
        );

        let refreshed_inventory = inventory([SessionBuilder::new("beta").window(
            WindowBuilder::new("beta:agents").pane(
                PaneBuilder::agent("beta:codex", HarnessKind::CodexCli)
                    .status(AgentStatus::NeedsAttention),
            ),
        )]);

        reduce(&mut state, Action::ReplaceInventory(refreshed_inventory));

        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.modal, None);
    }
}
