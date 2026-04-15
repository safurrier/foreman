use crate::app::action::{Action, DraftEdit, SelectionDirection};
use crate::app::state::{
    AppState, FlashNavigateKind, Focus, ModalState, Mode, OperatorAlert, OperatorAlertLevel,
    OperatorAlertSource, PaneId, SearchState, SelectionTarget, SessionId, WindowId,
};
use crate::integrations::stabilize_inventory;
use crate::services::notifications::{
    evaluate_inventory_notifications, NotificationDecision, NotificationPolicyContext,
    NotificationRequest,
};
use crate::services::pull_requests::PullRequestLookup;

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
    OpenBrowser {
        url: String,
    },
    CopyToClipboard {
        text: String,
    },
    Notify {
        request: NotificationRequest,
    },
    LogNotificationDecision {
        decision: NotificationDecision,
    },
    CycleTheme,
    Quit,
}

pub fn reduce(state: &mut AppState, action: Action) -> Vec<Effect> {
    match action {
        Action::RequestMode(mode) => {
            if mode.precedence() >= state.mode.precedence() {
                state.mode = mode;
                if mode == Mode::Input {
                    state.focus = Focus::Input;
                } else if mode == Mode::Help {
                    state.help_scroll = 0;
                }
            }
        }
        Action::BeginInput => {
            if state.selected_actionable_pane_id().is_some() {
                state.mode = Mode::Input;
                state.focus = Focus::Input;
            }
        }
        Action::BeginSearch => {
            state.mode = Mode::Search;
            state.focus = Focus::Sidebar;
            state.modal = None;
            state.flash = None;
            state.search = Some(SearchState::new(state.selection.clone()));
            reconcile_visible_selection(state);
        }
        Action::BeginFlash { kind } => {
            if state.base_visible_targets().is_empty() {
                return Vec::new();
            }

            state.mode = Mode::FlashNavigate;
            state.focus = Focus::Sidebar;
            state.modal = None;
            state.search = None;
            state.flash = Some(crate::app::FlashState::new(state.selection.clone(), kind));
            reconcile_flash_selection(state);
        }
        Action::SetPullRequestLookup {
            workspace_path,
            lookup,
        } => {
            state.pull_request_cache.insert(workspace_path, lookup);
            reconcile_pull_request_detail(state);
        }
        Action::SetRuntimeDiagnostics(diagnostics) => {
            state.runtime_diagnostics = diagnostics;
        }
        Action::SetSystemStats(snapshot) => {
            state.system_stats = snapshot;
        }
        Action::SetStartupError(startup_error) => {
            state.startup_error = startup_error;
        }
        Action::SetOperatorAlert(alert) => {
            state.operator_alert = alert;
        }
        Action::ToggleNotificationsMuted => {
            state.notifications.muted = !state.notifications.muted;
            state.notifications.last_status = Some(if state.notifications.muted {
                "Notifications muted".to_string()
            } else {
                "Notifications unmuted".to_string()
            });
        }
        Action::CycleNotificationProfile => {
            state.notifications.profile = state.notifications.profile.next();
            state.notifications.last_status = Some(format!(
                "Notification profile: {}",
                state.notifications.profile.label()
            ));
        }
        Action::TogglePullRequestDetail => {
            let Some(workspace_path) = state.selected_workspace_path() else {
                return Vec::new();
            };
            if state.selected_pull_request().is_none() {
                return Vec::new();
            }

            if state.is_pull_request_detail_open() {
                state.pull_request_detail_workspace = None;
                state.pull_request_detail_manual = false;
                state
                    .pull_request_auto_open_dismissed
                    .insert(workspace_path);
            } else {
                state.pull_request_detail_workspace = Some(workspace_path);
                state.pull_request_detail_manual = true;
            }
        }
        Action::CancelMode => {
            match state.mode {
                Mode::Input => {
                    state.input_draft.clear();
                }
                Mode::Search => {
                    let restore_selection = state
                        .search
                        .take()
                        .and_then(|search| search.restore_selection);
                    state.mode = Mode::Normal;
                    state.focus = Focus::Sidebar;
                    restore_base_selection(state, restore_selection);
                    reconcile_pull_request_detail(state);
                    return Vec::new();
                }
                Mode::FlashNavigate => {
                    let restore_selection =
                        state.flash.take().and_then(|flash| flash.restore_selection);
                    state.mode = Mode::Normal;
                    state.focus = Focus::Sidebar;
                    restore_base_selection(state, restore_selection);
                    reconcile_pull_request_detail(state);
                    return Vec::new();
                }
                Mode::Normal
                | Mode::PreviewScroll
                | Mode::Spawn
                | Mode::Rename
                | Mode::ConfirmKill => {}
                Mode::Help => {
                    state.help_scroll = 0;
                }
            }
            state.mode = Mode::Normal;
            state.focus = Focus::Sidebar;
            state.modal = None;
        }
        Action::OpenSelectedPullRequest => {
            if let Some(pull_request) = state.selected_pull_request() {
                return vec![Effect::OpenBrowser {
                    url: pull_request.url.clone(),
                }];
            }
        }
        Action::CopySelectedPullRequestUrl => {
            if let Some(pull_request) = state.selected_pull_request() {
                return vec![Effect::CopyToClipboard {
                    text: pull_request.url.clone(),
                }];
            }
        }
        Action::RequestQuit => {
            return vec![Effect::Quit];
        }
        Action::SetFocus(focus) => {
            state.focus = focus;
        }
        Action::ScrollHelpBy(delta) => {
            if state.mode == Mode::Help {
                if delta.is_negative() {
                    state.help_scroll = state.help_scroll.saturating_sub(delta.unsigned_abs());
                } else {
                    state.help_scroll = state.help_scroll.saturating_add(delta as u16);
                }
            }
        }
        Action::ScrollHelpToStart => {
            if state.mode == Mode::Help {
                state.help_scroll = 0;
            }
        }
        Action::ScrollHelpToEnd => {
            if state.mode == Mode::Help {
                state.help_scroll = u16::MAX;
            }
        }
        Action::SetSelection(target) => {
            if state.visible_targets().contains(&target) {
                state.selection = Some(target);
                state.focus = Focus::Sidebar;
                reconcile_pull_request_detail(state);
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
            reconcile_pull_request_detail(state);
        }
        Action::ReplaceInventory(mut inventory) => {
            let previous_inventory = state.inventory.clone();
            stabilize_inventory(&state.inventory, &mut inventory);
            state.inventory = inventory;
            state
                .collapsed_sessions
                .retain(|session_id| state.inventory.contains_session(session_id));
            state.reconcile_selection();
            reconcile_interaction_state(state);
            reconcile_pull_request_detail(state);
            return notification_effects_for_refresh(state, &previous_inventory);
        }
        Action::FocusSelectedPane => {
            if let Some(pane_id) = state.selected_actionable_pane_id() {
                return vec![Effect::FocusPane {
                    pane_id,
                    close_after: state.popup_mode,
                }];
            }
        }
        Action::CommitSearchSelection => {
            let selected = state.selection.clone();
            state.mode = Mode::Normal;
            state.search = None;

            match selected {
                Some(SelectionTarget::Session(session_id)) => {
                    state.collapsed_sessions.remove(&session_id);
                    reconcile_visible_selection(state);
                    reconcile_pull_request_detail(state);
                }
                Some(SelectionTarget::Pane(pane_id)) => {
                    return vec![Effect::FocusPane {
                        pane_id,
                        close_after: state.popup_mode,
                    }];
                }
                Some(SelectionTarget::Window(_)) | None => {}
            }
        }
        Action::OpenRenameModal {
            window_id,
            current_name,
        } => {
            if state.inventory.window(&window_id).is_some() {
                state.mode = Mode::Rename;
                state.focus = Focus::Input;
                state.modal = Some(ModalState::rename_window(window_id, current_name));
            }
        }
        Action::OpenSpawnModal { session_id } => {
            if state.inventory.contains_session(&session_id) {
                state.mode = Mode::Spawn;
                state.focus = Focus::Input;
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
            if state.mode == Mode::Search {
                if let Some(search) = state.search.as_mut() {
                    if !matches!(edit, DraftEdit::InsertNewline) {
                        apply_draft_edit(&mut search.draft, edit);
                        reconcile_visible_selection(state);
                        reconcile_pull_request_detail(state);
                    }
                }
            } else if state.mode == Mode::FlashNavigate {
                if !matches!(edit, DraftEdit::InsertNewline) {
                    return apply_flash_edit(state, edit);
                }
            } else if state.mode == Mode::Input {
                apply_draft_edit(&mut state.input_draft, edit);
            } else if let Some(draft) = state.modal.as_mut().and_then(ModalState::draft_mut) {
                apply_draft_edit(draft, edit);
            }
        }
        Action::SubmitActiveDraft => match state.mode {
            Mode::Input => {
                if let Some(pane_id) = state.selected_actionable_pane_id() {
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
            reconcile_visible_selection(state);
            reconcile_pull_request_detail(state);
        }
        Action::ToggleShowNonAgentPanes => {
            state.filters.show_non_agent_panes = !state.filters.show_non_agent_panes;
            state.reconcile_selection();
            reconcile_visible_selection(state);
            reconcile_pull_request_detail(state);
        }
        Action::CycleHarnessFilter => {
            let available_harnesses = state.inventory.available_harnesses();
            state.filters.cycle_harness(&available_harnesses);
            state.reconcile_selection();
            reconcile_visible_selection(state);
            reconcile_pull_request_detail(state);
        }
        Action::ToggleSessionCollapsed(session_id) => {
            if !state.inventory.contains_session(&session_id) {
                return Vec::new();
            }

            if !state.collapsed_sessions.insert(session_id.clone()) {
                state.collapsed_sessions.remove(&session_id);
            }

            state.reconcile_selection();
            reconcile_visible_selection(state);
            reconcile_pull_request_detail(state);
        }
        Action::SetSortMode(sort_mode) => {
            state.sort_mode = sort_mode;
            state.reconcile_selection();
            reconcile_visible_selection(state);
            reconcile_pull_request_detail(state);
        }
        Action::CycleTheme => {
            return vec![Effect::CycleTheme];
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

fn apply_flash_edit(state: &mut AppState, edit: DraftEdit) -> Vec<Effect> {
    let previous_input = state
        .flash
        .as_ref()
        .map(|flash| flash.draft.text.clone())
        .unwrap_or_default();

    if let Some(flash) = state.flash.as_mut() {
        apply_draft_edit(&mut flash.draft, edit);
    }

    let matches = state.matching_flash_targets();
    if matches.is_empty() {
        if let Some(flash) = state.flash.as_mut() {
            flash.draft = crate::app::TextDraft::from_text(previous_input);
        }
        reconcile_flash_selection(state);
        return Vec::new();
    }

    state.selection = Some(matches[0].target.clone());

    let input = state
        .flash
        .as_ref()
        .map(|flash| flash.draft.text.to_ascii_lowercase())
        .unwrap_or_default();
    let label_width = state
        .flash_targets()
        .first()
        .map(|candidate| candidate.label.len())
        .unwrap_or_default();
    if input.len() == label_width {
        if let Some(exact_match) = matches
            .into_iter()
            .find(|candidate| candidate.label == input)
        {
            return finish_flash_selection(state, exact_match.target);
        }
    }

    Vec::new()
}

fn finish_flash_selection(state: &mut AppState, target: SelectionTarget) -> Vec<Effect> {
    let kind = state
        .flash
        .take()
        .map(|flash| flash.kind)
        .unwrap_or(FlashNavigateKind::Jump);
    state.mode = Mode::Normal;
    state.selection = Some(target.clone());

    if kind == FlashNavigateKind::JumpAndFocus {
        if let SelectionTarget::Pane(pane_id) = target {
            return vec![Effect::FocusPane {
                pane_id,
                close_after: state.popup_mode,
            }];
        }
    }

    reconcile_pull_request_detail(state);

    Vec::new()
}

fn restore_base_selection(state: &mut AppState, restore_selection: Option<SelectionTarget>) {
    state.selection = state.inventory.reconcile_selection(
        restore_selection.as_ref(),
        &state.filters,
        &state.collapsed_sessions,
        state.sort_mode,
    );
}

fn reconcile_visible_selection(state: &mut AppState) {
    let visible_targets = state.visible_targets();
    if visible_targets.is_empty() {
        state.selection = None;
        return;
    }

    if state
        .selection
        .as_ref()
        .is_some_and(|selection| visible_targets.contains(selection))
    {
        return;
    }

    state.selection = Some(visible_targets[0].clone());
}

fn reconcile_flash_selection(state: &mut AppState) {
    let Some(flash) = state.flash.as_ref() else {
        return;
    };

    if flash.draft.text.is_empty() {
        restore_base_selection(state, flash.restore_selection.clone());
        reconcile_pull_request_detail(state);
        return;
    }

    let matches = state.matching_flash_targets();
    if let Some(target) = matches.first().map(|candidate| candidate.target.clone()) {
        state.selection = Some(target);
    } else {
        restore_base_selection(state, flash.restore_selection.clone());
    }

    reconcile_pull_request_detail(state);
}

fn reconcile_interaction_state(state: &mut AppState) {
    if state.mode == Mode::Input && state.selected_actionable_pane_id().is_none() {
        state.mode = Mode::Normal;
        state.focus = Focus::Sidebar;
        state.input_draft.clear();
    }

    if state.mode != Mode::Search {
        state.search = None;
    }

    if state.mode != Mode::FlashNavigate {
        state.flash = None;
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
        state.focus = Focus::Sidebar;
        state.modal = None;
    }

    if state.mode == Mode::Search {
        reconcile_visible_selection(state);
    }

    if state.mode == Mode::FlashNavigate {
        reconcile_flash_selection(state);
    }
}

fn reconcile_pull_request_detail(state: &mut AppState) {
    let Some(selected_workspace) = state.selected_workspace_path() else {
        state.pull_request_detail_workspace = None;
        state.pull_request_detail_manual = false;
        clear_pull_request_alert(state);
        return;
    };

    match state.pull_request_cache.get(&selected_workspace) {
        Some(PullRequestLookup::Available(_))
            if state.pull_request_detail_workspace.as_ref() == Some(&selected_workspace)
                && state.pull_request_detail_manual => {}
        Some(PullRequestLookup::Available(_)) => {
            clear_pull_request_alert(state);
            if state
                .pull_request_auto_open_dismissed
                .contains(&selected_workspace)
            {
                state.pull_request_detail_workspace = None;
                state.pull_request_detail_manual = false;
            } else {
                state.pull_request_detail_workspace = Some(selected_workspace);
                state.pull_request_detail_manual = false;
            }
        }
        Some(PullRequestLookup::Unavailable { message }) => {
            state.pull_request_detail_workspace = None;
            state.pull_request_detail_manual = false;
            state.operator_alert = Some(OperatorAlert::new(
                OperatorAlertSource::PullRequests,
                OperatorAlertLevel::Warn,
                format!("PR lookup unavailable: {message}"),
            ));
        }
        Some(PullRequestLookup::Unknown) | Some(PullRequestLookup::Missing) | None => {
            state.pull_request_detail_workspace = None;
            state.pull_request_detail_manual = false;
            clear_pull_request_alert(state);
        }
    }
}

fn clear_pull_request_alert(state: &mut AppState) {
    if state
        .operator_alert
        .as_ref()
        .is_some_and(|alert| alert.source == OperatorAlertSource::PullRequests)
    {
        state.operator_alert = None;
    }
}

fn notification_effects_for_refresh(
    state: &mut AppState,
    previous_inventory: &crate::app::Inventory,
) -> Vec<Effect> {
    state.notifications.refresh_tick = state.notifications.refresh_tick.saturating_add(1);
    state
        .notifications
        .cooldowns
        .retain(|key, _| state.inventory.pane(&key.pane_id).is_some());

    let decisions = evaluate_inventory_notifications(
        previous_inventory,
        &state.inventory,
        NotificationPolicyContext {
            selected_pane_id: state.selected_pane_id().as_ref(),
            muted: state.notifications.muted,
            profile: state.notifications.profile,
            refresh_tick: state.notifications.refresh_tick,
            cooldown_ticks: state.notifications.cooldown_ticks,
        },
        &state.notifications.cooldowns,
    );

    let mut effects = Vec::new();
    for decision in decisions {
        if let Some(request) = decision.request.clone() {
            state.notifications.cooldowns.insert(
                crate::app::NotificationCooldownKey {
                    pane_id: request.pane_id.clone(),
                    kind: request.kind,
                },
                state.notifications.refresh_tick,
            );
            effects.push(Effect::Notify { request });
        }
        effects.push(Effect::LogNotificationDecision { decision });
    }

    effects
}

#[cfg(test)]
mod tests {
    use crate::app::{
        inventory, reduce, Action, AgentStatus, AppState, DraftEdit, Effect, FlashNavigateKind,
        Focus, HarnessKind, ModalState, Mode, OperatorAlertLevel, OperatorAlertSource, PaneBuilder,
        SelectionDirection, SelectionTarget, SessionBuilder, SortMode, WindowBuilder,
    };
    use crate::services::pull_requests::{PullRequestData, PullRequestLookup, PullRequestStatus};
    use crate::services::system_stats::SystemStatsSnapshot;
    use std::path::PathBuf;

    fn sample_inventory() -> crate::app::Inventory {
        inventory([
            SessionBuilder::new("alpha").window(
                WindowBuilder::new("alpha:agents").pane(
                    PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                        .working_dir("/tmp/alpha")
                        .status(AgentStatus::Working)
                        .activity_score(10),
                ),
            ),
            SessionBuilder::new("beta").window(
                WindowBuilder::new("beta:agents").pane(
                    PaneBuilder::agent("beta:codex", HarnessKind::CodexCli)
                        .working_dir("/tmp/beta")
                        .status(AgentStatus::NeedsAttention)
                        .activity_score(4),
                ),
            ),
            SessionBuilder::new("notes").window(
                WindowBuilder::new("notes:window")
                    .pane(PaneBuilder::new("notes:pane").working_dir("/tmp/notes")),
            ),
        ])
    }

    fn sample_pull_request(number: u64, title: &str, url: &str) -> PullRequestLookup {
        PullRequestLookup::Available(PullRequestData {
            number,
            title: title.to_string(),
            url: url.to_string(),
            repository: "foreman".to_string(),
            branch: "feat/pr-awareness".to_string(),
            base_branch: "main".to_string(),
            author: "alex".to_string(),
            status: PullRequestStatus::Open,
        })
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
    fn harness_filter_cycles_and_reconciles_selection() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("beta:codex".into()));

        reduce(&mut state, Action::CycleHarnessFilter);
        assert_eq!(state.harness_filter_label(), "claude");
        assert_eq!(
            state.selection,
            Some(SelectionTarget::Session("alpha".into()))
        );

        reduce(&mut state, Action::CycleHarnessFilter);
        assert_eq!(state.harness_filter_label(), "codex");
        assert_eq!(
            state.selection,
            Some(SelectionTarget::Session("beta".into()))
        );

        reduce(&mut state, Action::CycleHarnessFilter);
        assert_eq!(state.harness_filter_label(), "all");
    }

    #[test]
    fn harness_filter_skips_empty_supported_harnesses() {
        let mut state = AppState::with_inventory(sample_inventory());

        reduce(&mut state, Action::CycleHarnessFilter);
        assert_eq!(state.harness_filter_label(), "claude");

        reduce(&mut state, Action::CycleHarnessFilter);
        assert_eq!(state.harness_filter_label(), "codex");

        reduce(&mut state, Action::CycleHarnessFilter);
        assert_eq!(state.harness_filter_label(), "all");
    }

    #[test]
    fn harness_filter_hides_non_agent_rows_even_when_shell_toggles_are_enabled() {
        let mut state = AppState::with_inventory(inventory([
            SessionBuilder::new("notes")
                .window(WindowBuilder::new("notes:window").pane(PaneBuilder::new("notes:pane"))),
            SessionBuilder::new("alpha").window(
                WindowBuilder::new("alpha:agents").pane(
                    PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                        .status(AgentStatus::Working)
                        .activity_score(3),
                ),
            ),
        ]));

        reduce(&mut state, Action::ToggleShowNonAgentSessions);
        reduce(&mut state, Action::ToggleShowNonAgentPanes);
        reduce(&mut state, Action::CycleHarnessFilter);

        assert!(!state
            .visible_targets()
            .contains(&SelectionTarget::Pane("notes:pane".into())));
        assert_eq!(state.harness_filter_label(), "claude");
    }

    #[test]
    fn sort_mode_change_preserves_logical_selection() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));

        reduce(&mut state, Action::SetSortMode(SortMode::AttentionFirst));

        assert_eq!(
            state.selection,
            Some(SelectionTarget::Pane("alpha:claude".into()))
        );
    }

    #[test]
    fn cancel_mode_resets_to_normal_sidebar() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.mode = Mode::Help;
        state.focus = Focus::Preview;
        state.help_scroll = 9;

        reduce(&mut state, Action::CancelMode);

        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.focus, Focus::Sidebar);
        assert_eq!(state.help_scroll, 0);
    }

    #[test]
    fn help_scroll_actions_are_reducer_owned_and_clamp() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.mode = Mode::Help;
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));

        reduce(&mut state, Action::ScrollHelpBy(3));
        assert_eq!(state.help_scroll, 3);
        assert_eq!(
            state.selection,
            Some(SelectionTarget::Pane("alpha:claude".into()))
        );

        reduce(&mut state, Action::ScrollHelpBy(-2));
        assert_eq!(state.help_scroll, 1);

        reduce(&mut state, Action::ScrollHelpBy(-8));
        assert_eq!(state.help_scroll, 0);

        reduce(&mut state, Action::ScrollHelpToEnd);
        assert_eq!(state.help_scroll, u16::MAX);

        reduce(&mut state, Action::RequestMode(Mode::Help));
        assert_eq!(state.help_scroll, 0);
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

    #[test]
    fn search_filters_matches_and_cancel_restores_previous_selection() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));

        reduce(&mut state, Action::BeginSearch);
        for ch in "codex".chars() {
            reduce(&mut state, Action::EditDraft(DraftEdit::InsertChar(ch)));
        }

        assert_eq!(state.mode, Mode::Search);
        assert_eq!(
            state
                .search
                .as_ref()
                .map(|search| search.draft.text.as_str()),
            Some("codex")
        );
        assert_eq!(
            state.visible_targets(),
            vec![SelectionTarget::Pane("beta:codex".into())]
        );
        assert_eq!(
            state.selection,
            Some(SelectionTarget::Pane("beta:codex".into()))
        );

        reduce(&mut state, Action::CancelMode);

        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.search, None);
        assert_eq!(
            state.selection,
            Some(SelectionTarget::Pane("alpha:claude".into()))
        );
    }

    #[test]
    fn search_navigation_moves_between_filtered_matches() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("beta:codex".into()));

        reduce(&mut state, Action::BeginSearch);
        for ch in "alpha".chars() {
            reduce(&mut state, Action::EditDraft(DraftEdit::InsertChar(ch)));
        }

        assert_eq!(
            state.visible_targets(),
            vec![
                SelectionTarget::Session("alpha".into()),
                SelectionTarget::Window("alpha:agents".into()),
                SelectionTarget::Pane("alpha:claude".into()),
            ]
        );
        assert_eq!(
            state.selection,
            Some(SelectionTarget::Session("alpha".into()))
        );

        reduce(&mut state, Action::MoveSelection(SelectionDirection::Next));
        assert_eq!(
            state.selection,
            Some(SelectionTarget::Window("alpha:agents".into()))
        );

        reduce(&mut state, Action::MoveSelection(SelectionDirection::Next));
        assert_eq!(
            state.selection,
            Some(SelectionTarget::Pane("alpha:claude".into()))
        );
    }

    #[test]
    fn search_commit_focuses_selected_pane() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));

        reduce(&mut state, Action::BeginSearch);
        for ch in "codex".chars() {
            reduce(&mut state, Action::EditDraft(DraftEdit::InsertChar(ch)));
        }

        let effects = reduce(&mut state, Action::CommitSearchSelection);

        assert_eq!(
            effects,
            vec![super::Effect::FocusPane {
                pane_id: "beta:codex".into(),
                close_after: false,
            }]
        );
        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.search, None);
    }

    #[test]
    fn flash_jump_only_selects_target_and_exits_mode() {
        let mut state = AppState::with_inventory(sample_inventory());

        reduce(
            &mut state,
            Action::BeginFlash {
                kind: FlashNavigateKind::Jump,
            },
        );
        let label = state
            .flash_label_for_target(&SelectionTarget::Pane("beta:codex".into()))
            .expect("flash label should exist");

        let mut effects = Vec::new();
        for ch in label.chars() {
            effects = reduce(&mut state, Action::EditDraft(DraftEdit::InsertChar(ch)));
        }

        assert!(effects.is_empty());
        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.flash, None);
        assert_eq!(
            state.selection,
            Some(SelectionTarget::Pane("beta:codex".into()))
        );
    }

    #[test]
    fn flash_jump_and_focus_emits_focus_effect() {
        let mut state = AppState::with_inventory(sample_inventory());

        reduce(
            &mut state,
            Action::BeginFlash {
                kind: FlashNavigateKind::JumpAndFocus,
            },
        );
        let label = state
            .flash_label_for_target(&SelectionTarget::Pane("alpha:claude".into()))
            .expect("flash label should exist");

        let mut effects = Vec::new();
        for ch in label.chars() {
            effects = reduce(&mut state, Action::EditDraft(DraftEdit::InsertChar(ch)));
        }

        assert_eq!(
            effects,
            vec![super::Effect::FocusPane {
                pane_id: "alpha:claude".into(),
                close_after: false,
            }]
        );
        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.flash, None);
    }

    #[test]
    fn flash_labels_expand_to_fixed_width_beyond_single_character_capacity() {
        let inventory = inventory([SessionBuilder::new("alpha").window({
            let mut window = WindowBuilder::new("alpha:agents");
            for index in 0..27 {
                window = window.pane(
                    PaneBuilder::agent(format!("alpha:pane:{index}"), HarnessKind::ClaudeCode)
                        .status(AgentStatus::Working)
                        .working_dir("/tmp/alpha")
                        .title(format!("pane-{index}")),
                );
            }
            window
        })]);
        let mut state = AppState::with_inventory(inventory);

        reduce(
            &mut state,
            Action::BeginFlash {
                kind: FlashNavigateKind::Jump,
            },
        );

        let labels = state.flash_targets();
        assert_eq!(
            labels.first().map(|candidate| candidate.label.as_str()),
            Some("aa")
        );
        assert_eq!(
            labels.get(25).map(|candidate| candidate.label.as_str()),
            Some("az")
        );
        assert_eq!(
            labels.get(26).map(|candidate| candidate.label.as_str()),
            Some("ba")
        );
    }

    #[test]
    fn pull_request_detail_auto_opens_once_and_close_suppresses_auto_reopen() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));

        reduce(
            &mut state,
            Action::SetPullRequestLookup {
                workspace_path: PathBuf::from("/tmp/alpha"),
                lookup: sample_pull_request(42, "Add PR awareness", "https://example.com/pr/42"),
            },
        );

        assert!(state.is_pull_request_detail_open());
        assert_eq!(
            state.pull_request_detail_workspace.as_ref(),
            Some(&PathBuf::from("/tmp/alpha"))
        );
        assert!(!state.pull_request_detail_manual);

        reduce(&mut state, Action::TogglePullRequestDetail);

        assert!(!state.is_pull_request_detail_open());
        assert!(state
            .pull_request_auto_open_dismissed
            .contains(&PathBuf::from("/tmp/alpha")));

        reduce(
            &mut state,
            Action::SetPullRequestLookup {
                workspace_path: PathBuf::from("/tmp/alpha"),
                lookup: sample_pull_request(42, "Add PR awareness", "https://example.com/pr/42"),
            },
        );

        assert!(!state.is_pull_request_detail_open());

        reduce(&mut state, Action::TogglePullRequestDetail);

        assert!(state.is_pull_request_detail_open());
        assert!(state.pull_request_detail_manual);

        reduce(
            &mut state,
            Action::SetPullRequestLookup {
                workspace_path: PathBuf::from("/tmp/alpha"),
                lookup: sample_pull_request(42, "Add PR awareness", "https://example.com/pr/42"),
            },
        );

        assert!(state.is_pull_request_detail_open());
        assert!(state.pull_request_detail_manual);
    }

    #[test]
    fn pull_request_open_and_copy_actions_emit_effects() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));
        reduce(
            &mut state,
            Action::SetPullRequestLookup {
                workspace_path: PathBuf::from("/tmp/alpha"),
                lookup: sample_pull_request(7, "Wire browser actions", "https://example.com/pr/7"),
            },
        );

        let open_effects = reduce(&mut state, Action::OpenSelectedPullRequest);
        let copy_effects = reduce(&mut state, Action::CopySelectedPullRequestUrl);

        assert_eq!(
            open_effects,
            vec![super::Effect::OpenBrowser {
                url: "https://example.com/pr/7".to_string(),
            }]
        );
        assert_eq!(
            copy_effects,
            vec![super::Effect::CopyToClipboard {
                text: "https://example.com/pr/7".to_string(),
            }]
        );
    }

    #[test]
    fn pull_request_unavailable_sets_operator_alert_for_selected_workspace() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));

        reduce(
            &mut state,
            Action::SetPullRequestLookup {
                workspace_path: PathBuf::from("/tmp/alpha"),
                lookup: PullRequestLookup::Unavailable {
                    message: "GitHub CLI is not installed".to_string(),
                },
            },
        );

        let alert = state
            .operator_alert
            .as_ref()
            .expect("pull request failure should surface an operator alert");
        assert_eq!(alert.source, OperatorAlertSource::PullRequests);
        assert_eq!(alert.level, OperatorAlertLevel::Warn);
        assert!(alert.message.contains("PR lookup unavailable"));
        assert!(alert.message.contains("GitHub CLI is not installed"));
    }

    #[test]
    fn pull_request_alert_clears_after_workspace_recovers() {
        let mut state = AppState::with_inventory(sample_inventory());
        state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));

        reduce(
            &mut state,
            Action::SetPullRequestLookup {
                workspace_path: PathBuf::from("/tmp/alpha"),
                lookup: PullRequestLookup::Unavailable {
                    message: "gh auth missing".to_string(),
                },
            },
        );
        assert!(state.operator_alert.is_some());

        reduce(
            &mut state,
            Action::SetPullRequestLookup {
                workspace_path: PathBuf::from("/tmp/alpha"),
                lookup: sample_pull_request(9, "Recover PR state", "https://example.com/pr/9"),
            },
        );

        assert!(state.operator_alert.is_none());
    }

    #[test]
    fn toggle_mute_and_cycle_profile_update_notification_state() {
        let mut state = AppState::with_inventory(sample_inventory());

        reduce(&mut state, Action::ToggleNotificationsMuted);
        assert!(state.notifications.muted);
        assert_eq!(
            state.notifications.last_status.as_deref(),
            Some("Notifications muted")
        );

        reduce(&mut state, Action::CycleNotificationProfile);
        assert_eq!(
            state.notifications.profile,
            crate::app::NotificationProfile::CompletionOnly
        );
        assert_eq!(
            state.notifications.last_status.as_deref(),
            Some("Notification profile: COMPLETE")
        );
    }

    #[test]
    fn runtime_snapshot_actions_update_observability_state() {
        let mut state = AppState::default();

        reduce(
            &mut state,
            Action::SetSystemStats(SystemStatsSnapshot {
                cpu_pressure_percent: Some(9),
                memory_pressure_percent: Some(51),
            }),
        );
        reduce(
            &mut state,
            Action::SetStartupError(Some("tmux unavailable".to_string())),
        );
        reduce(
            &mut state,
            Action::SetOperatorAlert(Some(crate::app::OperatorAlert::new(
                OperatorAlertSource::Tmux,
                OperatorAlertLevel::Warn,
                "tmux unavailable: failed to connect",
            ))),
        );

        assert_eq!(
            state.system_stats,
            SystemStatsSnapshot {
                cpu_pressure_percent: Some(9),
                memory_pressure_percent: Some(51),
            }
        );
        assert_eq!(state.startup_error.as_deref(), Some("tmux unavailable"));
        assert_eq!(
            state.operator_alert.as_ref().map(|alert| alert.source),
            Some(OperatorAlertSource::Tmux)
        );
    }

    #[test]
    fn replace_inventory_emits_completion_notification_and_records_cooldown() {
        let mut state = AppState::with_inventory(inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                    .working_dir("/tmp/alpha")
                    .status(AgentStatus::Working),
            ),
        )]));
        state.selection = Some(SelectionTarget::Session("alpha".into()));

        let refreshed_inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                    .working_dir("/tmp/alpha")
                    .status(AgentStatus::Idle),
            ),
        )]);

        let first_effects = reduce(
            &mut state,
            Action::ReplaceInventory(refreshed_inventory.clone()),
        );
        assert!(first_effects.is_empty());

        let effects = reduce(&mut state, Action::ReplaceInventory(refreshed_inventory));

        assert_eq!(effects.len(), 2);
        assert!(matches!(effects[0], super::Effect::Notify { .. }));
        assert!(matches!(
            effects[1],
            super::Effect::LogNotificationDecision { .. }
        ));
        assert!(state
            .notifications
            .cooldowns
            .contains_key(&crate::app::NotificationCooldownKey {
                pane_id: "alpha:claude".into(),
                kind: crate::app::NotificationKind::Completion,
            }));
    }

    #[test]
    fn replace_inventory_suppresses_notification_when_selected_or_muted() {
        let initial_inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                    .working_dir("/tmp/alpha")
                    .status(AgentStatus::Working),
            ),
        )]);
        let refreshed_inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                    .working_dir("/tmp/alpha")
                    .status(AgentStatus::Idle),
            ),
        )]);

        let mut selected_state = AppState::with_inventory(initial_inventory.clone());
        selected_state.selection = Some(SelectionTarget::Pane("alpha:claude".into()));
        let first_selected_effects = reduce(
            &mut selected_state,
            Action::ReplaceInventory(refreshed_inventory.clone()),
        );
        assert!(first_selected_effects.is_empty());
        let selected_effects = reduce(
            &mut selected_state,
            Action::ReplaceInventory(refreshed_inventory.clone()),
        );
        assert_eq!(selected_effects.len(), 1);
        assert!(matches!(
            &selected_effects[0],
            super::Effect::LogNotificationDecision { decision }
                if decision.request.is_none()
        ));

        let mut muted_state = AppState::with_inventory(initial_inventory);
        muted_state.selection = Some(SelectionTarget::Session("alpha".into()));
        muted_state.notifications.muted = true;
        let first_muted_effects = reduce(
            &mut muted_state,
            Action::ReplaceInventory(refreshed_inventory.clone()),
        );
        assert!(first_muted_effects.is_empty());
        let muted_effects = reduce(
            &mut muted_state,
            Action::ReplaceInventory(refreshed_inventory),
        );
        assert_eq!(muted_effects.len(), 1);
        assert!(matches!(
            &muted_effects[0],
            super::Effect::LogNotificationDecision { decision }
                if decision.request.is_none()
        ));
    }

    #[test]
    fn cycle_theme_emits_runtime_effect() {
        let mut state = AppState::with_inventory(sample_inventory());

        let effects = reduce(&mut state, Action::CycleTheme);

        assert_eq!(effects, vec![Effect::CycleTheme]);
    }
}
