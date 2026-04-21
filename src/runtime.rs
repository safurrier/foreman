use crate::adapters::tmux::{
    InventoryLoadMetrics, InventoryLoadPolicy, SystemTmuxBackend, TmuxAdapter,
};
use crate::app::{
    action_for_command, map_key_event, reduce, Action, AppState, DraftEdit, Effect, OperatorAlert,
    OperatorAlertLevel, OperatorAlertSource,
};
use crate::cli::PreparedBootstrap;
use crate::doctor::{
    primary_runtime_alert_finding, runtime_findings, DoctorSeverity, RuntimeDoctorContext,
};
use crate::integrations::{
    apply_configured_claude_signals, apply_configured_codex_signals, apply_configured_pi_signals,
    ClaudeNativeOverlaySummary, CodexNativeOverlaySummary, PiNativeOverlaySummary,
};
use crate::services::logging::RunLogger;
use crate::services::notifications::{
    build_notification_dispatcher, NotificationDispatcher, NotificationRequest,
};
use crate::services::pull_requests::{
    PullRequestLookup, PullRequestService, SystemPullRequestBackend,
};
use crate::services::system_stats::{SysinfoSystemStatsBackend, SystemStatsService};
use crate::ui::render::{render, sidebar_viewport_rows_for_area};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use std::collections::BTreeMap;
use std::fmt;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

const SELECTION_PULL_REQUEST_LOOKUP_DEBOUNCE_MS: u64 = 180;
const SLOW_RENDER_FRAME_MS: u128 = 33;
const SLOW_ACTION_MS: u128 = 40;
const SLOW_INVENTORY_REFRESH_MS: u128 = 200;
const SLOW_PULL_REQUEST_LOOKUP_MS: u128 = 75;
const OFFSCREEN_PREVIEW_REFRESH_BATCH: usize = 4;

pub(crate) fn run(prepared: PreparedBootstrap) -> Result<(), RuntimeError> {
    let _terminal_guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut runtime = DashboardRuntime::new(prepared);
    runtime.maybe_refresh_selected_pull_request(true, "bootstrap")?;
    runtime.run_loop(&mut terminal)
}

#[derive(Debug)]
pub enum RuntimeError {
    Io(io::Error),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<io::Error> for RuntimeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

struct DashboardRuntime {
    runtime: crate::config::RuntimeConfig,
    logger: RunLogger,
    state: AppState,
    tmux: TmuxAdapter<SystemTmuxBackend>,
    pull_requests: PullRequestService<SystemPullRequestBackend>,
    notifications: NotificationDispatcher,
    system_stats: SystemStatsService<SysinfoSystemStatsBackend>,
    last_pull_request_lookup: BTreeMap<PathBuf, Instant>,
    pending_pull_request_lookup: Option<PendingPullRequestLookup>,
    offscreen_capture_cursor: usize,
    inventory_refresh: InventoryRefreshWorker,
    notification_policy_epoch: u64,
}

#[derive(Debug, Clone)]
struct PendingPullRequestLookup {
    workspace_path: PathBuf,
    due_at: Instant,
}

struct InventoryRefreshWorker {
    request_tx: Sender<InventoryRefreshCommand>,
    result_rx: Receiver<InventoryRefreshResult>,
    next_generation: u64,
    in_flight_generation: Option<u64>,
    rerun_requested: bool,
}

enum InventoryRefreshCommand {
    Refresh(InventoryRefreshRequest),
    Shutdown,
}

struct InventoryRefreshRequest {
    generation: u64,
    previous_inventory: crate::app::Inventory,
    policy: InventoryLoadPolicy,
    notification_policy_epoch: u64,
}

struct InventoryRefreshResult {
    generation: u64,
    elapsed_ms: u128,
    notification_policy_epoch: u64,
    result: Result<InventoryRefreshPayload, String>,
}

type InventoryRefreshPayload = (
    crate::app::Inventory,
    InventoryLoadMetrics,
    NativeOverlayTiming,
    ClaudeNativeOverlaySummary,
    CodexNativeOverlaySummary,
    PiNativeOverlaySummary,
);

#[derive(Debug, Clone, Copy, Default)]
struct NativeOverlayTiming {
    claude_ms: u128,
    codex_ms: u128,
    pi_ms: u128,
    total_ms: u128,
}

impl DashboardRuntime {
    fn new(prepared: PreparedBootstrap) -> Self {
        let tmux = TmuxAdapter::new(SystemTmuxBackend::new(prepared.runtime.tmux_socket.clone()));
        let inventory_refresh = spawn_inventory_refresh_worker(prepared.runtime.clone());

        Self {
            tmux,
            pull_requests: PullRequestService::new(SystemPullRequestBackend::new()),
            notifications: build_notification_dispatcher(&prepared.runtime.notification_backends),
            system_stats: SystemStatsService::new(SysinfoSystemStatsBackend::new()),
            last_pull_request_lookup: BTreeMap::new(),
            pending_pull_request_lookup: None,
            offscreen_capture_cursor: 0,
            inventory_refresh,
            notification_policy_epoch: 0,
            runtime: prepared.runtime,
            logger: prepared.logger,
            state: prepared.state,
        }
    }

    fn run_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), RuntimeError> {
        let poll_interval = Duration::from_millis(self.runtime.poll_interval_ms.max(1));
        let mut last_inventory_refresh = Instant::now();
        let initial_size = terminal.size()?;

        self.logger.info("dashboard_started")?;
        self.logger.debug("dashboard_event_loop_started")?;
        self.apply_runtime_action(Action::SetSidebarViewportRows(
            sidebar_viewport_rows_for_area(Rect {
                x: 0,
                y: 0,
                width: initial_size.width,
                height: initial_size.height,
            }),
        ))?;

        loop {
            self.drain_inventory_refresh_results()?;
            let current_size = terminal.size()?;
            let sidebar_rows = sidebar_viewport_rows_for_area(Rect {
                x: 0,
                y: 0,
                width: current_size.width,
                height: current_size.height,
            }) as usize;
            if self.state.sidebar_viewport_rows != sidebar_rows {
                self.apply_runtime_action(Action::SetSidebarViewportRows(sidebar_rows as u16))?;
            }

            let render_started = Instant::now();
            terminal.draw(|frame| render(frame, &self.state, self.runtime.theme))?;
            let render_elapsed_ms = render_started.elapsed().as_millis();
            self.logger.log_timing(
                "render_frame",
                &format!(
                    "elapsed_ms={render_elapsed_ms} width={} height={} visible_entries={} selected_index={} sidebar_scroll={} viewport_rows={} mode={} focus={}",
                    current_size.width,
                    current_size.height,
                    self.state.visible_target_count(),
                    selected_index_field(&self.state),
                    self.state.sidebar_scroll(),
                    self.state.sidebar_viewport_rows,
                    self.state.mode_label().to_ascii_lowercase(),
                    self.state.focus_label().to_ascii_lowercase(),
                ),
            )?;
            self.maybe_log_slow_operation(
                "render_frame",
                SLOW_RENDER_FRAME_MS,
                render_elapsed_ms,
                &format!(
                    "elapsed_ms={render_elapsed_ms} width={} height={} visible_entries={} selected_index={} sidebar_scroll={} viewport_rows={}",
                    current_size.width,
                    current_size.height,
                    self.state.visible_target_count(),
                    selected_index_field(&self.state),
                    self.state.sidebar_scroll(),
                    self.state.sidebar_viewport_rows,
                ),
            )?;

            let timeout = self
                .next_wait_timeout(poll_interval.saturating_sub(last_inventory_refresh.elapsed()));
            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) => {
                        if let Some(command) = map_key_event(key, self.state.focus, self.state.mode)
                        {
                            let previous_workspace = self.state.selected_workspace_path();
                            let action = action_for_command(&self.state, command);
                            if self.apply_action(action)? {
                                break;
                            }
                            self.schedule_selected_pull_request_refresh(previous_workspace);
                        }
                    }
                    Event::Paste(text) => {
                        self.apply_paste(text)?;
                    }
                    Event::Resize(width, height) => {
                        self.apply_runtime_action(Action::SetSidebarViewportRows(
                            sidebar_viewport_rows_for_area(Rect {
                                x: 0,
                                y: 0,
                                width,
                                height,
                            }),
                        ))?;
                    }
                    Event::Mouse(_) | Event::FocusGained | Event::FocusLost => {}
                }
            }

            self.run_pending_pull_request_refresh()?;

            if last_inventory_refresh.elapsed() >= poll_interval {
                self.request_inventory_refresh()?;
                self.refresh_system_stats()?;
                last_inventory_refresh = Instant::now();
            }
        }

        self.logger.info("dashboard_stopped")?;
        Ok(())
    }

    fn apply_paste(&mut self, text: String) -> Result<(), RuntimeError> {
        for ch in text.chars() {
            let action = match ch {
                '\n' | '\r' => Action::EditDraft(DraftEdit::InsertNewline),
                other => Action::EditDraft(DraftEdit::InsertChar(other)),
            };
            let _ = self.apply_action(action)?;
        }

        Ok(())
    }

    fn apply_action(&mut self, action: Action) -> Result<bool, RuntimeError> {
        let action_label = action.label();
        let action_started = Instant::now();
        let reduce_started = Instant::now();
        let effects = reduce(&mut self.state, action);
        let reduce_ms = reduce_started.elapsed().as_millis();
        let effect_count = effects.len();
        let effects_started = Instant::now();
        let should_quit = self.execute_effects(effects)?;
        let effects_ms = effects_started.elapsed().as_millis();
        let total_ms = action_started.elapsed().as_millis();
        if matches!(
            action_label,
            "toggle-notifications-muted" | "cycle-notification-profile"
        ) {
            self.notification_policy_epoch = self.notification_policy_epoch.saturating_add(1);
        }

        self.logger.log_timing(
            "action",
            &format!(
                "action={action_label} reduce_ms={reduce_ms} effects={effect_count} effects_ms={effects_ms} total_ms={total_ms}{}",
                self.action_trace_context(action_label),
            ),
        )?;
        self.maybe_log_slow_operation(
            "action",
            SLOW_ACTION_MS,
            total_ms,
            &format!(
                "action={action_label} total_ms={total_ms}{}",
                self.action_trace_context(action_label),
            ),
        )?;

        Ok(should_quit)
    }

    fn apply_runtime_action(&mut self, action: Action) -> Result<(), RuntimeError> {
        let _ = self.apply_action(action)?;
        Ok(())
    }

    fn execute_effects(&mut self, effects: Vec<Effect>) -> Result<bool, RuntimeError> {
        let mut should_quit = false;

        for effect in effects {
            match effect {
                Effect::FocusPane {
                    pane_id,
                    close_after,
                } => match self.tmux.focus_pane(&pane_id) {
                    Ok(()) => {
                        self.clear_alert_source(OperatorAlertSource::Tmux)?;
                        if close_after {
                            should_quit = true;
                        }
                    }
                    Err(error) => {
                        self.record_alert(
                            OperatorAlertSource::Tmux,
                            OperatorAlertLevel::Warn,
                            format!("Focus failed: {error}"),
                        )?;
                    }
                },
                Effect::SendInput { pane_id, text } => {
                    match self.tmux.send_input(&pane_id, &text) {
                        Ok(_) => {
                            self.clear_alert_source(OperatorAlertSource::Tmux)?;
                        }
                        Err(error) => {
                            self.record_alert(
                                OperatorAlertSource::Tmux,
                                OperatorAlertLevel::Warn,
                                format!("Input send failed: {error}"),
                            )?;
                        }
                    }
                }
                Effect::RenameWindow { window_id, name } => {
                    match self.tmux.rename_window(&window_id, &name) {
                        Ok(_) => {
                            self.clear_alert_source(OperatorAlertSource::Tmux)?;
                            self.request_inventory_refresh()?;
                        }
                        Err(error) => {
                            self.record_alert(
                                OperatorAlertSource::Tmux,
                                OperatorAlertLevel::Warn,
                                format!("Rename failed: {error}"),
                            )?;
                        }
                    }
                }
                Effect::SpawnWindow {
                    session_id,
                    command,
                } => match self.tmux.spawn_window(&session_id, &command) {
                    Ok(_) => {
                        self.clear_alert_source(OperatorAlertSource::Tmux)?;
                        self.request_inventory_refresh()?;
                    }
                    Err(error) => {
                        self.record_alert(
                            OperatorAlertSource::Tmux,
                            OperatorAlertLevel::Warn,
                            format!("Spawn failed: {error}"),
                        )?;
                    }
                },
                Effect::KillPane { pane_id } => match self.tmux.kill_pane(&pane_id) {
                    Ok(_) => {
                        self.clear_alert_source(OperatorAlertSource::Tmux)?;
                        self.request_inventory_refresh()?;
                    }
                    Err(error) => {
                        self.record_alert(
                            OperatorAlertSource::Tmux,
                            OperatorAlertLevel::Warn,
                            format!("Kill failed: {error}"),
                        )?;
                    }
                },
                Effect::OpenBrowser { url } => match self.pull_requests.open_in_browser(&url) {
                    Ok(()) => {
                        self.clear_alert_source(OperatorAlertSource::Browser)?;
                    }
                    Err(error) => {
                        self.record_alert(
                            OperatorAlertSource::Browser,
                            OperatorAlertLevel::Warn,
                            format!("Open in browser failed: {error}"),
                        )?;
                    }
                },
                Effect::CopyToClipboard { text } => {
                    match self.pull_requests.copy_to_clipboard(&text) {
                        Ok(()) => {
                            self.clear_alert_source(OperatorAlertSource::Clipboard)?;
                        }
                        Err(error) => {
                            self.record_alert(
                                OperatorAlertSource::Clipboard,
                                OperatorAlertLevel::Warn,
                                format!("Clipboard copy failed: {error}"),
                            )?;
                        }
                    }
                }
                Effect::Notify { request } => {
                    self.dispatch_notification(&request)?;
                }
                Effect::LogNotificationDecision { decision } => {
                    self.logger.log_notification_decision(&decision)?;
                }
                Effect::CycleTheme => {
                    self.runtime.theme = self.runtime.theme.next();
                }
                Effect::Quit => {
                    should_quit = true;
                }
            }
        }

        Ok(should_quit)
    }

    fn apply_inventory_refresh_result(
        &mut self,
        payload: InventoryRefreshPayload,
        elapsed_ms: u128,
    ) -> Result<(), RuntimeError> {
        let (inventory, tmux_metrics, native_timing, claude_native, codex_native, pi_native) =
            payload;
        let outcome = "loaded";
        self.apply_runtime_action(Action::SetStartupError(None))?;
        self.clear_alert_source(OperatorAlertSource::Tmux)?;
        self.apply_runtime_action(Action::ReplaceInventory(inventory))?;

        let summary = self.state.inventory_summary();
        self.logger.log_timing(
            "inventory_tmux",
            &format!(
                "pane_records={} captures={} priority_captures={} deferred_captures={} forced_captures={} reused_previews={} capture_failures={} list_panes_ms={} capture_total_ms={} capture_max_ms={} total_ms={}",
                tmux_metrics.pane_records,
                tmux_metrics.capture_count,
                tmux_metrics.priority_capture_count,
                tmux_metrics.deferred_capture_count,
                tmux_metrics.forced_capture_count,
                tmux_metrics.reused_preview_count,
                tmux_metrics.capture_failures,
                tmux_metrics.list_panes_ms,
                tmux_metrics.capture_total_ms,
                tmux_metrics.capture_max_ms,
                tmux_metrics.total_ms,
            ),
        )?;
        for pane_id in &tmux_metrics.capture_failure_panes {
            self.logger.log_tmux_capture_failure(pane_id.as_str())?;
        }
        self.logger.log_timing(
            "inventory_native",
            &format!(
                "claude_ms={} codex_ms={} pi_ms={} total_ms={} claude_applied={} claude_fallbacks={} codex_applied={} codex_fallbacks={} pi_applied={} pi_fallbacks={}",
                native_timing.claude_ms,
                native_timing.codex_ms,
                native_timing.pi_ms,
                native_timing.total_ms,
                claude_native.applied,
                claude_native.fallback_to_compatibility,
                codex_native.applied,
                codex_native.fallback_to_compatibility,
                pi_native.applied,
                pi_native.fallback_to_compatibility,
            ),
        )?;
        self.logger.log_inventory(&summary)?;
        self.logger.log_claude_native_summary(&claude_native)?;
        for warning in &claude_native.warnings {
            self.logger.log_claude_native_warning(warning)?;
        }
        self.logger.log_codex_native_summary(&codex_native)?;
        for warning in &codex_native.warnings {
            self.logger.log_codex_native_warning(warning)?;
        }
        self.logger.log_pi_native_summary(&pi_native)?;
        for warning in &pi_native.warnings {
            self.logger.log_pi_native_warning(warning)?;
        }
        self.refresh_runtime_diagnostics(&claude_native, &codex_native, &pi_native)?;

        self.maybe_refresh_selected_pull_request(false, "inventory-refresh")?;
        self.logger.log_timing(
            "inventory_refresh",
            &format!("outcome={outcome} elapsed_ms={elapsed_ms}"),
        )?;
        self.maybe_log_slow_operation(
            "inventory_refresh",
            SLOW_INVENTORY_REFRESH_MS,
            elapsed_ms,
            &format!("outcome={outcome} elapsed_ms={elapsed_ms}"),
        )?;
        Ok(())
    }

    fn handle_inventory_refresh_error(
        &mut self,
        error_message: String,
        elapsed_ms: u128,
    ) -> Result<(), RuntimeError> {
        self.apply_runtime_action(Action::SetStartupError(Some(error_message.clone())))?;
        self.logger.log_tmux_error(&error_message)?;
        self.record_alert(
            OperatorAlertSource::Tmux,
            OperatorAlertLevel::Warn,
            format!("tmux unavailable: {error_message}"),
        )?;
        self.logger.log_timing(
            "inventory_refresh",
            &format!("outcome=error elapsed_ms={elapsed_ms}"),
        )?;
        self.maybe_log_slow_operation(
            "inventory_refresh",
            SLOW_INVENTORY_REFRESH_MS,
            elapsed_ms,
            &format!("outcome=error elapsed_ms={elapsed_ms}"),
        )?;
        Ok(())
    }

    fn refresh_runtime_diagnostics(
        &mut self,
        claude_native: &ClaudeNativeOverlaySummary,
        codex_native: &CodexNativeOverlaySummary,
        pi_native: &PiNativeOverlaySummary,
    ) -> Result<(), RuntimeError> {
        let diagnostics = runtime_findings(&RuntimeDoctorContext {
            runtime: &self.runtime,
            config_exists: self.runtime.config_file.exists(),
            inventory: &self.state.inventory,
            claude_native,
            codex_native,
            pi_native,
        });
        self.apply_runtime_action(Action::SetRuntimeDiagnostics(diagnostics.clone()))?;

        let existing_non_diagnostic_alert = self
            .state
            .operator_alert
            .as_ref()
            .is_some_and(|alert| alert.source != OperatorAlertSource::Diagnostics);
        if existing_non_diagnostic_alert {
            return Ok(());
        }

        if let Some(finding) = primary_runtime_alert_finding(&diagnostics) {
            let level = match finding.severity {
                DoctorSeverity::Error => OperatorAlertLevel::Error,
                DoctorSeverity::Warn => OperatorAlertLevel::Warn,
                DoctorSeverity::Ok | DoctorSeverity::Info => OperatorAlertLevel::Info,
            };
            self.record_alert(
                OperatorAlertSource::Diagnostics,
                level,
                finding.summary.clone(),
            )?;
        } else {
            self.clear_alert_source(OperatorAlertSource::Diagnostics)?;
        }

        Ok(())
    }

    fn inventory_capture_policy(&mut self) -> InventoryLoadPolicy {
        let priority_panes = self.state.refresh_priority_pane_ids();
        let all_panes = self.state.inventory.pane_ids();
        let offscreen_panes = all_panes
            .into_iter()
            .filter(|pane_id| !priority_panes.contains(pane_id))
            .collect::<Vec<_>>();

        let deferred_panes =
            rotating_capture_batch(&offscreen_panes, &mut self.offscreen_capture_cursor);

        InventoryLoadPolicy {
            priority_panes,
            deferred_panes,
        }
    }

    fn request_inventory_refresh(&mut self) -> Result<(), RuntimeError> {
        if self.inventory_refresh.in_flight_generation.is_some() {
            self.inventory_refresh.rerun_requested = true;
            return Ok(());
        }

        let generation = self.inventory_refresh.next_generation;
        self.inventory_refresh.next_generation += 1;
        let request = InventoryRefreshRequest {
            generation,
            previous_inventory: self.state.inventory.clone(),
            policy: self.inventory_capture_policy(),
            notification_policy_epoch: self.notification_policy_epoch,
        };
        self.logger.debug("inventory_refresh_started")?;
        self.inventory_refresh
            .request_tx
            .send(InventoryRefreshCommand::Refresh(request))
            .map_err(|error| worker_channel_error(error.to_string()))?;
        self.inventory_refresh.in_flight_generation = Some(generation);
        Ok(())
    }

    fn drain_inventory_refresh_results(&mut self) -> Result<(), RuntimeError> {
        loop {
            let result = match self.inventory_refresh.result_rx.try_recv() {
                Ok(result) => result,
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => {
                    return Err(worker_channel_error("inventory refresh worker stopped"))
                }
            };

            if self.inventory_refresh.in_flight_generation == Some(result.generation) {
                self.inventory_refresh.in_flight_generation = None;
            }

            if result.notification_policy_epoch != self.notification_policy_epoch {
                self.inventory_refresh.rerun_requested = false;
                self.request_inventory_refresh()?;
                continue;
            }

            match result.result {
                Ok(payload) => self.apply_inventory_refresh_result(payload, result.elapsed_ms)?,
                Err(error_message) => {
                    self.handle_inventory_refresh_error(error_message, result.elapsed_ms)?
                }
            }

            if self.inventory_refresh.rerun_requested
                && self.inventory_refresh.in_flight_generation.is_none()
            {
                self.inventory_refresh.rerun_requested = false;
                self.request_inventory_refresh()?;
            }
        }
    }

    fn refresh_system_stats(&mut self) -> Result<(), RuntimeError> {
        let snapshot = self.system_stats.snapshot().unwrap_or_default();
        self.logger.log_system_stats(&snapshot)?;
        self.apply_runtime_action(Action::SetSystemStats(snapshot))?;
        Ok(())
    }

    fn maybe_refresh_selected_pull_request(
        &mut self,
        force: bool,
        trigger: &'static str,
    ) -> Result<(), RuntimeError> {
        if !self.runtime.pull_request_monitoring_enabled {
            self.pending_pull_request_lookup = None;
            return Ok(());
        }

        let Some(workspace_path) = self.state.selected_workspace_path() else {
            self.pending_pull_request_lookup = None;
            return Ok(());
        };

        if !self.pull_request_lookup_due(&workspace_path, force) {
            return Ok(());
        }

        let lookup_started = Instant::now();
        let lookup = match self.pull_requests.lookup(&workspace_path) {
            Ok(lookup) => lookup,
            Err(error) => PullRequestLookup::Unavailable {
                message: error.to_string(),
            },
        };

        self.last_pull_request_lookup
            .insert(workspace_path.clone(), Instant::now());
        let elapsed_ms = lookup_started.elapsed().as_millis();
        self.logger.log_timing(
            "pull_request_lookup",
            &format!(
                "trigger={trigger} workspace={} outcome={} elapsed_ms={elapsed_ms}",
                workspace_path.display(),
                pull_request_lookup_outcome_label(&lookup),
            ),
        )?;
        self.maybe_log_slow_operation(
            "pull_request_lookup",
            SLOW_PULL_REQUEST_LOOKUP_MS,
            elapsed_ms,
            &format!(
                "trigger={trigger} workspace={} outcome={} elapsed_ms={elapsed_ms}",
                workspace_path.display(),
                pull_request_lookup_outcome_label(&lookup),
            ),
        )?;
        self.logger
            .log_pull_request_lookup(&workspace_path, &lookup)?;
        if self
            .pending_pull_request_lookup
            .as_ref()
            .is_some_and(|pending| pending.workspace_path == workspace_path)
        {
            self.pending_pull_request_lookup = None;
        }
        let previous_pull_request_alert = self
            .state
            .operator_alert
            .as_ref()
            .filter(|alert| alert.source == OperatorAlertSource::PullRequests)
            .cloned();
        self.apply_runtime_action(Action::SetPullRequestLookup {
            workspace_path,
            lookup,
        })?;
        self.log_pull_request_alert_transition(previous_pull_request_alert)?;
        Ok(())
    }

    fn dispatch_notification(&mut self, request: &NotificationRequest) -> Result<(), RuntimeError> {
        match self.notifications.dispatch(request) {
            Ok(receipt) => {
                self.logger
                    .log_notification_backend_selected(request, &receipt)?;
                self.clear_alert_source(OperatorAlertSource::Notifications)?;
            }
            Err(error) => {
                self.logger
                    .log_notification_backend_failure(request, &error)?;
                self.record_alert(
                    OperatorAlertSource::Notifications,
                    OperatorAlertLevel::Warn,
                    format!("Notification delivery failed: {error}"),
                )?;
            }
        }

        Ok(())
    }

    fn clear_alert_source(&mut self, source: OperatorAlertSource) -> Result<(), RuntimeError> {
        if self
            .state
            .operator_alert
            .as_ref()
            .is_some_and(|alert| alert.source == source)
        {
            self.apply_runtime_action(Action::SetOperatorAlert(None))?;
        }

        Ok(())
    }

    fn record_alert(
        &mut self,
        source: OperatorAlertSource,
        level: OperatorAlertLevel,
        message: String,
    ) -> Result<(), RuntimeError> {
        let alert = OperatorAlert::new(source, level, message);
        self.logger.log_operator_alert(&alert)?;
        self.apply_runtime_action(Action::SetOperatorAlert(Some(alert)))?;
        Ok(())
    }

    fn log_pull_request_alert_transition(
        &mut self,
        previous_alert: Option<OperatorAlert>,
    ) -> Result<(), RuntimeError> {
        let current_alert = self
            .state
            .operator_alert
            .as_ref()
            .filter(|alert| alert.source == OperatorAlertSource::PullRequests)
            .cloned();
        if current_alert != previous_alert {
            if let Some(alert) = current_alert.as_ref() {
                self.logger.log_operator_alert(alert)?;
            }
        }

        Ok(())
    }

    fn next_wait_timeout(&self, inventory_timeout: Duration) -> Duration {
        let pending_timeout = self
            .pending_pull_request_lookup
            .as_ref()
            .map(|pending| pending.due_at.saturating_duration_since(Instant::now()))
            .unwrap_or(inventory_timeout);
        inventory_timeout.min(pending_timeout)
    }

    fn schedule_selected_pull_request_refresh(&mut self, previous_workspace: Option<PathBuf>) {
        let current_workspace = self.state.selected_workspace_path();
        if current_workspace == previous_workspace {
            return;
        }

        if !self.runtime.pull_request_monitoring_enabled {
            self.pending_pull_request_lookup = None;
            return;
        }

        let Some(workspace_path) = current_workspace else {
            self.pending_pull_request_lookup = None;
            return;
        };

        if !self.pull_request_lookup_due(&workspace_path, false) {
            self.pending_pull_request_lookup = None;
            return;
        }

        self.pending_pull_request_lookup = Some(PendingPullRequestLookup {
            workspace_path,
            due_at: Instant::now()
                + Duration::from_millis(SELECTION_PULL_REQUEST_LOOKUP_DEBOUNCE_MS),
        });
    }

    fn run_pending_pull_request_refresh(&mut self) -> Result<(), RuntimeError> {
        let Some(pending) = self.pending_pull_request_lookup.clone() else {
            return Ok(());
        };

        if pending.due_at > Instant::now() {
            return Ok(());
        }

        let Some(selected_workspace) = self.state.selected_workspace_path() else {
            self.pending_pull_request_lookup = None;
            return Ok(());
        };

        if selected_workspace != pending.workspace_path {
            self.pending_pull_request_lookup = None;
            self.schedule_selected_pull_request_refresh(Some(pending.workspace_path));
            return Ok(());
        }

        self.pending_pull_request_lookup = None;
        self.maybe_refresh_selected_pull_request(false, "selection-idle")
    }

    fn pull_request_lookup_due(&self, workspace_path: &PathBuf, force: bool) -> bool {
        let poll_interval =
            Duration::from_millis(self.runtime.pull_request_poll_interval_ms.max(1));
        force
            || !self.state.pull_request_cache.contains_key(workspace_path)
            || self
                .last_pull_request_lookup
                .get(workspace_path)
                .is_none_or(|last_lookup| last_lookup.elapsed() >= poll_interval)
    }

    fn maybe_log_slow_operation(
        &mut self,
        operation: &str,
        threshold_ms: u128,
        elapsed_ms: u128,
        fields: &str,
    ) -> Result<(), RuntimeError> {
        if elapsed_ms > threshold_ms {
            self.logger
                .log_slow_operation(operation, threshold_ms, fields)?;
        }
        Ok(())
    }

    fn action_trace_context(&self, action_label: &str) -> String {
        match action_label {
            "move-selection" | "set-selection" | "set-sidebar-viewport-rows" => format!(
                " visible_entries={} selected_index={} selected_target={} sidebar_scroll={} viewport_rows={} sort={} filter={} mode={} focus={}",
                self.state.visible_target_count(),
                selected_index_field(&self.state),
                selected_target_field(&self.state),
                self.state.sidebar_scroll(),
                self.state.sidebar_viewport_rows,
                self.state.sort_label(),
                self.state.harness_filter_label(),
                self.state.mode_label().to_ascii_lowercase(),
                self.state.focus_label().to_ascii_lowercase(),
            ),
            _ => String::new(),
        }
    }
}

impl Drop for InventoryRefreshWorker {
    fn drop(&mut self) {
        let _ = self.request_tx.send(InventoryRefreshCommand::Shutdown);
    }
}

fn rotating_capture_batch(
    pane_ids: &[crate::app::PaneId],
    cursor: &mut usize,
) -> std::collections::BTreeSet<crate::app::PaneId> {
    if pane_ids.is_empty() {
        *cursor = 0;
        return std::collections::BTreeSet::new();
    }

    let budget = OFFSCREEN_PREVIEW_REFRESH_BATCH.min(pane_ids.len());
    let start = (*cursor).min(pane_ids.len()) % pane_ids.len();
    let batch = (0..budget)
        .map(|offset| pane_ids[(start + offset) % pane_ids.len()].clone())
        .collect();
    *cursor = (start + budget) % pane_ids.len();
    batch
}

fn spawn_inventory_refresh_worker(runtime: crate::config::RuntimeConfig) -> InventoryRefreshWorker {
    let (request_tx, request_rx) = mpsc::channel::<InventoryRefreshCommand>();
    let (result_tx, result_rx) = mpsc::channel::<InventoryRefreshResult>();

    thread::Builder::new()
        .name("foreman-inventory-refresh".to_string())
        .spawn(move || {
            let tmux = TmuxAdapter::new(SystemTmuxBackend::new(runtime.tmux_socket.clone()));
            while let Ok(command) = request_rx.recv() {
                match command {
                    InventoryRefreshCommand::Refresh(request) => {
                        let refresh_started = Instant::now();
                        let result = load_inventory_refresh_payload(
                            &tmux,
                            &runtime,
                            &request.previous_inventory,
                            &request.policy,
                        );
                        let _ = result_tx.send(InventoryRefreshResult {
                            generation: request.generation,
                            elapsed_ms: refresh_started.elapsed().as_millis(),
                            notification_policy_epoch: request.notification_policy_epoch,
                            result,
                        });
                    }
                    InventoryRefreshCommand::Shutdown => break,
                }
            }
        })
        .expect("inventory refresh worker should spawn");

    InventoryRefreshWorker {
        request_tx,
        result_rx,
        next_generation: 0,
        in_flight_generation: None,
        rerun_requested: false,
    }
}

fn load_inventory_refresh_payload(
    tmux: &TmuxAdapter<SystemTmuxBackend>,
    runtime: &crate::config::RuntimeConfig,
    previous_inventory: &crate::app::Inventory,
    policy: &InventoryLoadPolicy,
) -> Result<InventoryRefreshPayload, String> {
    let (mut inventory, tmux_metrics) = tmux
        .load_inventory_profiled_with_policy(
            runtime.capture_lines,
            Some(previous_inventory),
            policy,
        )
        .map_err(|error| error.to_string())?;

    let native_started = Instant::now();
    let claude_started = Instant::now();
    let claude_native = apply_configured_claude_signals(
        &mut inventory,
        runtime.claude_native_dir.as_deref(),
        runtime.claude_integration_preference,
    );
    let claude_ms = claude_started.elapsed().as_millis();
    let codex_started = Instant::now();
    let codex_native = apply_configured_codex_signals(
        &mut inventory,
        runtime.codex_native_dir.as_deref(),
        runtime.codex_integration_preference,
    );
    let codex_ms = codex_started.elapsed().as_millis();
    let pi_started = Instant::now();
    let pi_native = apply_configured_pi_signals(
        &mut inventory,
        runtime.pi_native_dir.as_deref(),
        runtime.pi_integration_preference,
    );
    let pi_ms = pi_started.elapsed().as_millis();

    Ok((
        inventory,
        tmux_metrics,
        NativeOverlayTiming {
            claude_ms,
            codex_ms,
            pi_ms,
            total_ms: native_started.elapsed().as_millis(),
        },
        claude_native,
        codex_native,
        pi_native,
    ))
}

fn worker_channel_error(message: impl Into<String>) -> RuntimeError {
    RuntimeError::Io(io::Error::other(message.into()))
}

fn pull_request_lookup_outcome_label(lookup: &PullRequestLookup) -> &'static str {
    match lookup {
        PullRequestLookup::Unknown => "unknown",
        PullRequestLookup::Missing => "missing",
        PullRequestLookup::Available(_) => "available",
        PullRequestLookup::Unavailable { .. } => "unavailable",
    }
}

fn selected_index_field(state: &AppState) -> String {
    state
        .selected_visible_index()
        .map(|index| index.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn selected_target_field(state: &AppState) -> &'static str {
    match state.selection.as_ref() {
        Some(crate::app::SelectionTarget::Session(_)) => "session",
        Some(crate::app::SelectionTarget::Window(_)) => "window",
        Some(crate::app::SelectionTarget::Pane(_)) => "pane",
        None => "none",
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self, RuntimeError> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, Show, LeaveAlternateScreen);
    }
}
