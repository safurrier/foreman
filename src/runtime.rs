use crate::adapters::tmux::{
    InventoryLoadMetrics, InventoryLoadPolicy, SystemTmuxBackend, TmuxAdapter,
};
use crate::app::{
    action_for_command, map_key_event, reduce, Action, AppState, DraftEdit, Effect, OperatorAlert,
    OperatorAlertLevel, OperatorAlertSource, PaneId, PaneKey,
};
use crate::cli::PreparedBootstrap;
use crate::doctor::{
    primary_runtime_alert_finding, runtime_findings, DoctorSeverity, RuntimeDoctorContext,
};
use crate::integrations::{
    apply_configured_claude_signals, apply_configured_codex_signals, apply_configured_pi_signals,
    ClaudeNativeOverlaySummary, CodexNativeOverlaySummary, PiNativeOverlaySummary,
};
use crate::services::control_api::{AgentEntry, AgentsResponse};
use crate::services::extensions::{collect_workspace_extensions, ControlExtensionCard};
use crate::services::logging::RunLogger;
use crate::services::notifications::{
    build_notification_dispatcher, NotificationDispatcher, NotificationRequest,
};
use crate::services::pull_requests::{
    PullRequestLookup, PullRequestService, SystemPullRequestBackend,
};
use crate::services::startup_cache::{current_time_ms, write_startup_cache};
use crate::services::system_stats::{SysinfoSystemStatsBackend, SystemStatsService};
use crate::services::ui_preferences::{save_ui_preferences, PersistedUiPreferences};
use crate::sources::{
    ForemanSource, SnapshotSource, SourceConfig, SourceDescriptor, SourceDiagnostic, SourceScope,
    SshSource,
};
use crate::ui::render::{render, sidebar_viewport_rows_for_area_with_popup};
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
use std::fs;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

const SELECTION_PULL_REQUEST_LOOKUP_DEBOUNCE_MS: u64 = 180;
const SELECTION_EXTENSION_LOOKUP_DEBOUNCE_MS: u64 = 180;
const SLOW_RENDER_FRAME_MS: u128 = 33;
const SLOW_ACTION_MS: u128 = 40;
const SLOW_INVENTORY_REFRESH_MS: u128 = 200;
const SLOW_PULL_REQUEST_LOOKUP_MS: u128 = 75;
const SLOW_STARTUP_CACHE_WRITE_MS: u128 = 20;
const OFFSCREEN_PREVIEW_REFRESH_BATCH: usize = 4;
const STARTUP_CACHE_WRITE_INTERVAL_MS: u64 = 5_000;
const PULL_REQUEST_LOOKUP_RESULT_POLL_MS: u64 = 50;
const EXTENSION_LOOKUP_RESULT_POLL_MS: u64 = 50;
const UI_PREFERENCES_WRITE_DEBOUNCE_MS: u64 = 300;
const POPUP_INPUT_POLL_MAX_MS: u64 = 5;
const POPUP_SOURCE_REFRESH_MIN_INTERVAL_MS: u64 = 60_000;
const POPUP_BACKGROUND_MERGE_IDLE_MS: u64 = 150;
const POPUP_READY_INPUT_DRAIN_LIMIT: usize = 32;

pub(crate) fn run(prepared: PreparedBootstrap) -> Result<(), RuntimeError> {
    let _terminal_guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut runtime = DashboardRuntime::new(prepared);
    runtime.maybe_refresh_selected_pull_request(true, "bootstrap")?;
    runtime.maybe_refresh_selected_extensions(true, "bootstrap")?;
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
    pull_request_lookup: PullRequestLookupWorker,
    extension_lookup: ExtensionLookupWorker,
    notifications: NotificationDispatcher,
    system_stats: SystemStatsService<SysinfoSystemStatsBackend>,
    last_pull_request_lookup: BTreeMap<PathBuf, Instant>,
    pending_pull_request_lookup: Option<PendingPullRequestLookup>,
    last_extension_lookup: BTreeMap<PathBuf, Instant>,
    pending_extension_lookup: Option<PendingExtensionLookup>,
    offscreen_capture_cursor: usize,
    inventory_refresh: InventoryRefreshWorker,
    notification_policy_epoch: u64,
    startup_cache: StartupCacheTracker,
    pending_selection_restore: Option<crate::app::SelectionTarget>,
    ui_preferences_dirty_since: Option<Instant>,
    persistent_runtime_diagnostics: Vec<crate::doctor::DoctorFinding>,
    last_input_at: Option<Instant>,
    deferred_inventory_refresh: Option<InventoryRefreshResult>,
}

#[derive(Debug, Clone)]
struct PendingPullRequestLookup {
    workspace_path: PathBuf,
    due_at: Instant,
}

#[derive(Debug, Clone)]
struct PendingExtensionLookup {
    workspace_path: PathBuf,
    due_at: Instant,
}

struct PullRequestLookupWorker {
    request_tx: Sender<PullRequestLookupCommand>,
    result_rx: Receiver<PullRequestLookupResult>,
    next_generation: u64,
    in_flight_generation: Option<u64>,
}

enum PullRequestLookupCommand {
    Lookup(PullRequestLookupRequest),
    Shutdown,
}

struct PullRequestLookupRequest {
    generation: u64,
    workspace_path: PathBuf,
    trigger: &'static str,
}

struct PullRequestLookupResult {
    generation: u64,
    workspace_path: PathBuf,
    trigger: &'static str,
    elapsed_ms: u128,
    lookup: PullRequestLookup,
}

struct ExtensionLookupWorker {
    request_tx: Sender<ExtensionLookupCommand>,
    result_rx: Receiver<ExtensionLookupResult>,
    next_generation: u64,
    in_flight_generation: Option<u64>,
}

enum ExtensionLookupCommand {
    Lookup(ExtensionLookupRequest),
    Shutdown,
}

struct ExtensionLookupRequest {
    generation: u64,
    workspace_path: PathBuf,
    pane_key: Option<PaneKey>,
    config_file: PathBuf,
    trigger: &'static str,
}

struct ExtensionLookupResult {
    generation: u64,
    workspace_path: PathBuf,
    trigger: &'static str,
    elapsed_ms: u128,
    cards: Vec<ControlExtensionCard>,
}

struct InventoryRefreshWorker {
    request_tx: Sender<InventoryRefreshCommand>,
    result_rx: Receiver<InventoryRefreshResult>,
    next_generation: u64,
    in_flight_generation: Option<u64>,
    rerun_requested: bool,
}

#[derive(Debug, Clone, Default)]
struct StartupCacheTracker {
    last_written_at_ms: Option<u64>,
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
    complete: bool,
    result: Result<InventoryRefreshPayload, String>,
}

type InventoryRefreshPayload = (
    crate::app::Inventory,
    InventoryLoadMetrics,
    NativeOverlayTiming,
    ClaudeNativeOverlaySummary,
    CodexNativeOverlaySummary,
    PiNativeOverlaySummary,
    Vec<SourceDiagnostic>,
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
        let tmux = TmuxAdapter::new(SystemTmuxBackend::with_target(
            prepared.runtime.tmux_target(),
        ));
        let inventory_refresh = spawn_inventory_refresh_worker(prepared.runtime.clone());
        let pull_request_lookup = spawn_pull_request_lookup_worker();
        let extension_lookup = spawn_extension_lookup_worker();
        let startup_cache = StartupCacheTracker {
            last_written_at_ms: prepared.startup_cache_generated_at_ms,
        };

        Self {
            tmux,
            pull_requests: PullRequestService::new(SystemPullRequestBackend::new()),
            pull_request_lookup,
            extension_lookup,
            notifications: build_notification_dispatcher(
                &prepared.runtime.notification_backends,
                &prepared.runtime.notification_sound_profile,
                &prepared.runtime.notification_sound_profiles,
                prepared.runtime.config_file.parent(),
                prepared.runtime.tmux_socket.clone(),
                Some(prepared.runtime.log_dir.join("latest.log")),
            ),
            system_stats: SystemStatsService::new(SysinfoSystemStatsBackend::new()),
            last_pull_request_lookup: BTreeMap::new(),
            pending_pull_request_lookup: None,
            last_extension_lookup: BTreeMap::new(),
            pending_extension_lookup: None,
            offscreen_capture_cursor: 0,
            inventory_refresh,
            notification_policy_epoch: 0,
            startup_cache,
            pending_selection_restore: prepared.pending_selection_restore,
            ui_preferences_dirty_since: None,
            persistent_runtime_diagnostics: prepared.persistent_runtime_diagnostics,
            last_input_at: None,
            deferred_inventory_refresh: None,
            runtime: prepared.runtime,
            logger: prepared.logger,
            state: prepared.state,
        }
    }

    fn run_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), RuntimeError> {
        let mut poll_interval = Duration::from_millis(self.runtime.poll_interval_ms.max(1));
        if self.state.popup_mode
            && (self.runtime.source.is_some()
                || self.runtime.sources.default_scope == SourceScope::All)
        {
            poll_interval =
                poll_interval.max(Duration::from_millis(POPUP_SOURCE_REFRESH_MIN_INTERVAL_MS));
        }
        let mut last_inventory_refresh = Instant::now();
        let mut initial_inventory_requested = false;
        let initial_size = terminal.size()?;

        self.logger.info("dashboard_started")?;
        self.logger.debug("dashboard_event_loop_started")?;
        self.apply_runtime_action(Action::SetSidebarViewportRows(
            sidebar_viewport_rows_for_area_with_popup(
                Rect {
                    x: 0,
                    y: 0,
                    width: initial_size.width,
                    height: initial_size.height,
                },
                self.state.popup_mode,
            ),
        ))?;

        loop {
            let (input_processed, should_quit) =
                self.drain_ready_input_events(POPUP_READY_INPUT_DRAIN_LIMIT)?;
            if should_quit {
                self.maybe_flush_ui_preferences(true, "shutdown")?;
                break;
            }

            if !input_processed {
                self.apply_deferred_inventory_refresh_if_idle()?;
                if initial_inventory_requested {
                    self.drain_inventory_refresh_results()?;
                }
                self.drain_pull_request_lookup_results()?;
                self.drain_extension_lookup_results()?;
            }
            self.maybe_flush_ui_preferences(false, "debounce")?;
            let current_size = terminal.size()?;
            let sidebar_rows = sidebar_viewport_rows_for_area_with_popup(
                Rect {
                    x: 0,
                    y: 0,
                    width: current_size.width,
                    height: current_size.height,
                },
                self.state.popup_mode,
            ) as usize;
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

            if !initial_inventory_requested {
                self.request_inventory_refresh()?;
                self.refresh_system_stats()?;
                last_inventory_refresh = Instant::now();
                initial_inventory_requested = true;
                self.drain_inventory_refresh_results()?;
                continue;
            }

            let timeout = self
                .next_wait_timeout(poll_interval.saturating_sub(last_inventory_refresh.elapsed()));
            if event::poll(timeout)? && self.handle_terminal_event(event::read()?)? {
                self.maybe_flush_ui_preferences(true, "shutdown")?;
                break;
            }

            if !input_processed {
                self.run_pending_pull_request_refresh()?;
                self.run_pending_extension_refresh()?;
                self.drain_pull_request_lookup_results()?;
                self.drain_extension_lookup_results()?;
            }

            if last_inventory_refresh.elapsed() >= poll_interval {
                self.request_inventory_refresh()?;
                self.refresh_system_stats()?;
                last_inventory_refresh = Instant::now();
            }
        }

        self.maybe_flush_ui_preferences(true, "shutdown")?;
        self.logger.info("dashboard_stopped")?;
        Ok(())
    }

    fn drain_ready_input_events(&mut self, limit: usize) -> Result<(bool, bool), RuntimeError> {
        let mut processed = false;
        for _ in 0..limit {
            if !event::poll(Duration::ZERO)? {
                break;
            }
            processed = true;
            if self.handle_terminal_event(event::read()?)? {
                return Ok((processed, true));
            }
        }
        Ok((processed, false))
    }

    fn handle_terminal_event(&mut self, event: Event) -> Result<bool, RuntimeError> {
        match event {
            Event::Key(key) => {
                self.last_input_at = Some(Instant::now());
                if let Some(command) = map_key_event(key, self.state.focus, self.state.mode) {
                    let previous_workspace = self.state.selected_workspace_path();
                    let action = action_for_command(&self.state, command);
                    if self.apply_action(action)? {
                        return Ok(true);
                    }
                    self.schedule_selected_pull_request_refresh(previous_workspace.clone());
                    self.schedule_selected_extension_refresh(previous_workspace);
                }
            }
            Event::Paste(text) => {
                self.last_input_at = Some(Instant::now());
                self.apply_paste(text)?;
            }
            Event::Resize(width, height) => {
                self.apply_runtime_action(Action::SetSidebarViewportRows(
                    sidebar_viewport_rows_for_area_with_popup(
                        Rect {
                            x: 0,
                            y: 0,
                            width,
                            height,
                        },
                        self.state.popup_mode,
                    ),
                ))?;
            }
            Event::Mouse(_) | Event::FocusGained | Event::FocusLost => {}
        }
        Ok(false)
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
        if should_persist_ui_preferences(action_label) {
            self.mark_ui_preferences_dirty();
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

    fn focus_pane_key(&self, pane_key: &PaneKey) -> Result<(), String> {
        if pane_key.is_local() {
            return self
                .tmux
                .focus_pane(&pane_key.pane_id)
                .map_err(|error| error.to_string());
        }
        self.run_source_action_command("focus", pane_key, None)
    }

    fn send_input_to_pane_key(&self, pane_key: &PaneKey, text: &str) -> Result<(), String> {
        if pane_key.is_local() {
            return self
                .tmux
                .send_input(&pane_key.pane_id, text)
                .map(|_| ())
                .map_err(|error| error.to_string());
        }
        self.run_source_action_command("send", pane_key, Some(text))
    }

    fn activate_source_display(&self, pane_key: &PaneKey) -> Result<(), String> {
        if pane_key.is_local() {
            return Ok(());
        }
        let source_id = crate::sources::SourceId::new(pane_key.source_id.as_str());
        let Some(source_config) = self.runtime.sources.get(&source_id) else {
            return Ok(());
        };
        let Some(jump) = source_config.jump() else {
            return Ok(());
        };
        let Some(command_template) = jump.activation_command.as_deref() else {
            return Ok(());
        };
        if command_template.trim().is_empty() {
            return Ok(());
        }

        let command = expand_activation_command(command_template, pane_key);
        let output = Command::new("sh")
            .arg("-lc")
            .arg(&command)
            .env("FOREMAN_SOURCE_ID", pane_key.source_id.as_str())
            .env("FOREMAN_PANE_ID", pane_key.pane_id.as_str())
            .output()
            .map_err(|error| format!("failed to run activation command: {error}"))?;
        if output.status.success() {
            return Ok(());
        }
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        if detail.is_empty() {
            Err(format!(
                "activation command exited with status {}",
                output.status
            ))
        } else {
            Err(detail)
        }
    }

    fn run_source_action_command(
        &self,
        command_name: &str,
        pane_key: &PaneKey,
        stdin_text: Option<&str>,
    ) -> Result<(), String> {
        let exe = std::env::current_exe().map_err(|error| error.to_string())?;
        let mut command = Command::new(exe);
        command
            .arg("--config-file")
            .arg(&self.runtime.config_file)
            .arg("--source")
            .arg(pane_key.source_id.as_str())
            .arg(command_name)
            .arg("--pane")
            .arg(pane_key.pane_id.as_str())
            .arg("--json");
        if stdin_text.is_some() {
            command.arg("--stdin").stdin(std::process::Stdio::piped());
        }
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        let mut child = command.spawn().map_err(|error| error.to_string())?;
        if let Some(text) = stdin_text {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                stdin
                    .write_all(text.as_bytes())
                    .map_err(|error| error.to_string())?;
            }
        }
        let output = child
            .wait_with_output()
            .map_err(|error| error.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                format!("{command_name} failed for {}", pane_key.stable_id())
            } else {
                stderr
            })
        }
    }

    fn execute_effects(&mut self, effects: Vec<Effect>) -> Result<bool, RuntimeError> {
        let mut should_quit = false;

        for effect in effects {
            match effect {
                Effect::FocusPane {
                    pane_key,
                    close_after,
                } => match self.focus_pane_key(&pane_key) {
                    Ok(()) => match self.activate_source_display(&pane_key) {
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
                                format!("Focused pane, but terminal activation failed: {error}"),
                            )?;
                        }
                    },
                    Err(error) => {
                        self.record_alert(
                            OperatorAlertSource::Tmux,
                            OperatorAlertLevel::Warn,
                            format!("Focus failed: {error}"),
                        )?;
                    }
                },
                Effect::SendInput { pane_key, text } => {
                    match self.send_input_to_pane_key(&pane_key, &text) {
                        Ok(_) => {
                            self.clear_alert_source(OperatorAlertSource::Tmux)?;
                        }
                        Err(error) => {
                            self.apply_runtime_action(Action::RestoreFailedInput {
                                pane_id: pane_key.pane_id,
                                text,
                            })?;
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
                            self.apply_runtime_action(Action::RestoreFailedRename {
                                window_id,
                                name,
                            })?;
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
                        self.apply_runtime_action(Action::RestoreFailedSpawn {
                            session_id,
                            command,
                        })?;
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
                Effect::RefreshPullRequest { workspace_path } => {
                    self.request_pull_request_lookup(workspace_path, "manual")?;
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
        complete: bool,
    ) -> Result<(), RuntimeError> {
        let (
            inventory,
            tmux_metrics,
            native_timing,
            claude_native,
            codex_native,
            pi_native,
            source_diagnostics,
        ) = payload;
        let outcome = "loaded";
        let was_startup_loading = self.state.startup_loading;
        let should_prime_priority_previews = was_startup_loading && !inventory.sessions.is_empty();
        self.apply_runtime_action(Action::SetStartupLoading(false))?;
        self.apply_runtime_action(Action::SetStartupCacheAge(None))?;
        self.apply_runtime_action(Action::SetStartupError(None))?;
        self.apply_runtime_action(Action::SetSourceDiagnostics(source_diagnostics.clone()))?;
        if source_diagnostics.is_empty() {
            self.clear_alert_source(OperatorAlertSource::Diagnostics)?;
        } else {
            self.record_alert(
                OperatorAlertSource::Diagnostics,
                OperatorAlertLevel::Warn,
                format!("{} source diagnostic(s)", source_diagnostics.len()),
            )?;
        }
        self.clear_alert_source(OperatorAlertSource::Tmux)?;
        self.apply_runtime_action(Action::ReplaceInventory(inventory))?;
        self.restore_pending_selection()?;

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
        self.maybe_write_startup_cache(complete)?;
        self.refresh_runtime_diagnostics(&claude_native, &codex_native, &pi_native)?;

        self.maybe_refresh_selected_pull_request(false, "inventory-refresh")?;
        self.maybe_refresh_selected_extensions(false, "inventory-refresh")?;
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
        if should_prime_priority_previews {
            self.request_inventory_refresh()?;
        }
        Ok(())
    }

    fn handle_inventory_refresh_error(
        &mut self,
        error_message: String,
        elapsed_ms: u128,
    ) -> Result<(), RuntimeError> {
        self.apply_runtime_action(Action::SetStartupLoading(false))?;
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
        let mut diagnostics = runtime_findings(&RuntimeDoctorContext {
            runtime: &self.runtime,
            config_exists: self.runtime.config_file.exists(),
            inventory: &self.state.inventory,
            claude_native,
            codex_native,
            pi_native,
        });
        diagnostics.extend(self.persistent_runtime_diagnostics.clone());
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
        let priority_panes: std::collections::BTreeSet<PaneId> = self
            .state
            .refresh_priority_pane_keys()
            .into_iter()
            .filter(|pane_key| pane_key.is_local())
            .map(|pane_key| pane_key.pane_id)
            .collect();
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
            capture_unseen_panes: false,
            capture_empty_previews: false,
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

            if result.complete
                && self.inventory_refresh.in_flight_generation == Some(result.generation)
            {
                self.inventory_refresh.in_flight_generation = None;
            }

            if result.notification_policy_epoch != self.notification_policy_epoch {
                self.inventory_refresh.rerun_requested = false;
                self.request_inventory_refresh()?;
                continue;
            }

            if self.should_defer_inventory_refresh_result(&result) {
                self.logger.debug(&format!(
                    "inventory_refresh_deferred generation={} elapsed_ms={} complete={}",
                    result.generation, result.elapsed_ms, result.complete
                ))?;
                self.deferred_inventory_refresh = Some(result);
                continue;
            }

            self.apply_inventory_refresh_result_message(result)?;

            if self.inventory_refresh.rerun_requested
                && self.inventory_refresh.in_flight_generation.is_none()
            {
                self.inventory_refresh.rerun_requested = false;
                self.request_inventory_refresh()?;
            }
        }
    }

    fn should_defer_inventory_refresh_result(&self, result: &InventoryRefreshResult) -> bool {
        self.state.popup_mode
            && (self.runtime.source.is_some()
                || self.runtime.sources.default_scope == SourceScope::All)
            && result.complete
            && result.result.is_ok()
            && self.last_input_at.is_some_and(|input_at| {
                input_at.elapsed() < Duration::from_millis(POPUP_BACKGROUND_MERGE_IDLE_MS)
            })
    }

    fn apply_deferred_inventory_refresh_if_idle(&mut self) -> Result<(), RuntimeError> {
        let should_apply = self.deferred_inventory_refresh.is_some()
            && self.last_input_at.is_none_or(|input_at| {
                input_at.elapsed() >= Duration::from_millis(POPUP_BACKGROUND_MERGE_IDLE_MS)
            });
        if should_apply {
            if let Some(result) = self.deferred_inventory_refresh.take() {
                self.logger.debug(&format!(
                    "inventory_refresh_deferred_apply generation={} elapsed_ms={} complete={}",
                    result.generation, result.elapsed_ms, result.complete
                ))?;
                self.apply_inventory_refresh_result_message(result)?;
            }
        }
        Ok(())
    }

    fn apply_inventory_refresh_result_message(
        &mut self,
        result: InventoryRefreshResult,
    ) -> Result<(), RuntimeError> {
        match result.result {
            Ok(payload) => {
                self.apply_inventory_refresh_result(payload, result.elapsed_ms, result.complete)
            }
            Err(error_message) => {
                self.handle_inventory_refresh_error(error_message, result.elapsed_ms)
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
        if self.state.popup_mode && trigger != "manual" {
            self.clear_pending_pull_request_lookup()?;
            return Ok(());
        }
        if !self.runtime.pull_request_monitoring_enabled {
            self.clear_pending_pull_request_lookup()?;
            return Ok(());
        }

        let Some(workspace_path) = self.state.selected_workspace_path() else {
            self.clear_pending_pull_request_lookup()?;
            return Ok(());
        };

        if !self.pull_request_lookup_due(&workspace_path, force) {
            return Ok(());
        }

        self.request_pull_request_lookup(workspace_path, trigger)
    }

    fn request_pull_request_lookup(
        &mut self,
        workspace_path: PathBuf,
        trigger: &'static str,
    ) -> Result<(), RuntimeError> {
        if self.pull_request_lookup.in_flight_generation.is_some() {
            self.replace_pending_pull_request_lookup(PendingPullRequestLookup {
                workspace_path: workspace_path.clone(),
                due_at: Instant::now()
                    + Duration::from_millis(SELECTION_PULL_REQUEST_LOOKUP_DEBOUNCE_MS),
            })?;
            self.apply_runtime_action(Action::SetPullRequestRefreshing {
                workspace_path,
                refreshing: true,
            })?;
            return Ok(());
        }

        let generation = self.pull_request_lookup.next_generation;
        self.pull_request_lookup.next_generation += 1;
        self.pull_request_lookup.in_flight_generation = Some(generation);
        self.apply_runtime_action(Action::SetPullRequestRefreshing {
            workspace_path: workspace_path.clone(),
            refreshing: true,
        })?;
        let send_result =
            self.pull_request_lookup
                .request_tx
                .send(PullRequestLookupCommand::Lookup(PullRequestLookupRequest {
                    generation,
                    workspace_path: workspace_path.clone(),
                    trigger,
                }));
        if let Err(error) = send_result {
            self.pull_request_lookup.in_flight_generation = None;
            self.apply_runtime_action(Action::SetPullRequestRefreshing {
                workspace_path,
                refreshing: false,
            })?;
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, error.to_string()).into());
        }

        Ok(())
    }

    fn drain_pull_request_lookup_results(&mut self) -> Result<(), RuntimeError> {
        loop {
            match self.pull_request_lookup.result_rx.try_recv() {
                Ok(result) => self.apply_pull_request_lookup_result(result)?,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }

        Ok(())
    }

    fn maybe_refresh_selected_extensions(
        &mut self,
        force: bool,
        trigger: &'static str,
    ) -> Result<(), RuntimeError> {
        if self.state.popup_mode && trigger != "manual" {
            self.clear_pending_extension_lookup()?;
            return Ok(());
        }
        let Some(workspace_path) = self.state.selected_workspace_path() else {
            self.clear_pending_extension_lookup()?;
            return Ok(());
        };

        if !self.extension_lookup_due(&workspace_path, force) {
            return Ok(());
        }

        let pane_key = self.state.selected_actionable_pane_key();
        self.request_extension_lookup(workspace_path, pane_key, trigger)
    }

    fn request_extension_lookup(
        &mut self,
        workspace_path: PathBuf,
        pane_key: Option<PaneKey>,
        trigger: &'static str,
    ) -> Result<(), RuntimeError> {
        if self.extension_lookup.in_flight_generation.is_some() {
            self.replace_pending_extension_lookup(PendingExtensionLookup {
                workspace_path: workspace_path.clone(),
                due_at: Instant::now()
                    + Duration::from_millis(SELECTION_EXTENSION_LOOKUP_DEBOUNCE_MS),
            })?;
            self.apply_runtime_action(Action::SetExtensionRefreshing {
                workspace_path,
                refreshing: true,
            })?;
            return Ok(());
        }

        let generation = self.extension_lookup.next_generation;
        self.extension_lookup.next_generation += 1;
        self.extension_lookup.in_flight_generation = Some(generation);
        self.apply_runtime_action(Action::SetExtensionRefreshing {
            workspace_path: workspace_path.clone(),
            refreshing: true,
        })?;
        let send_result = self
            .extension_lookup
            .request_tx
            .send(ExtensionLookupCommand::Lookup(ExtensionLookupRequest {
                generation,
                workspace_path: workspace_path.clone(),
                pane_key,
                config_file: self.runtime.config_file.clone(),
                trigger,
            }));
        if let Err(error) = send_result {
            self.extension_lookup.in_flight_generation = None;
            self.apply_runtime_action(Action::SetExtensionRefreshing {
                workspace_path,
                refreshing: false,
            })?;
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, error.to_string()).into());
        }

        Ok(())
    }

    fn drain_extension_lookup_results(&mut self) -> Result<(), RuntimeError> {
        loop {
            match self.extension_lookup.result_rx.try_recv() {
                Ok(result) => self.apply_extension_lookup_result(result)?,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }

        Ok(())
    }

    fn apply_extension_lookup_result(
        &mut self,
        result: ExtensionLookupResult,
    ) -> Result<(), RuntimeError> {
        if self.extension_lookup.in_flight_generation != Some(result.generation) {
            return Ok(());
        }

        self.extension_lookup.in_flight_generation = None;
        let ExtensionLookupResult {
            workspace_path,
            trigger,
            elapsed_ms,
            cards,
            ..
        } = result;
        self.last_extension_lookup
            .insert(workspace_path.clone(), Instant::now());
        self.logger.log_timing(
            "extension_lookup",
            &format!(
                "trigger={trigger} workspace={} cards={} elapsed_ms={elapsed_ms}",
                workspace_path.display(),
                cards.len(),
            ),
        )?;
        self.apply_runtime_action(Action::SetExtensionRefreshing {
            workspace_path: workspace_path.clone(),
            refreshing: false,
        })?;
        self.apply_runtime_action(Action::SetExtensionCards {
            workspace_path,
            cards,
        })?;
        Ok(())
    }

    fn apply_pull_request_lookup_result(
        &mut self,
        result: PullRequestLookupResult,
    ) -> Result<(), RuntimeError> {
        if self.pull_request_lookup.in_flight_generation != Some(result.generation) {
            return Ok(());
        }

        self.pull_request_lookup.in_flight_generation = None;
        let PullRequestLookupResult {
            workspace_path,
            trigger,
            elapsed_ms,
            lookup,
            ..
        } = result;

        self.last_pull_request_lookup
            .insert(workspace_path.clone(), Instant::now());
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
        self.apply_runtime_action(Action::SetPullRequestRefreshing {
            workspace_path: workspace_path.clone(),
            refreshing: false,
        })?;
        self.apply_runtime_action(Action::SetPullRequestLookup {
            workspace_path,
            lookup,
        })?;
        self.log_pull_request_alert_transition(previous_pull_request_alert)?;
        Ok(())
    }

    fn restore_pending_selection(&mut self) -> Result<(), RuntimeError> {
        let Some(target) = self.pending_selection_restore.clone() else {
            return Ok(());
        };
        if self.state.inventory.contains_target(&target) {
            self.apply_runtime_action(Action::SetSelection(target))?;
            self.pending_selection_restore = None;
        } else if !self.state.inventory.sessions.is_empty() {
            self.pending_selection_restore = None;
        }
        Ok(())
    }

    fn mark_ui_preferences_dirty(&mut self) {
        if self.ui_preferences_dirty_since.is_none() {
            self.ui_preferences_dirty_since = Some(Instant::now());
        }
    }

    fn ui_preferences_flush_timeout(&self, fallback: Duration) -> Duration {
        let Some(dirty_since) = self.ui_preferences_dirty_since else {
            return fallback;
        };
        Duration::from_millis(UI_PREFERENCES_WRITE_DEBOUNCE_MS)
            .saturating_sub(dirty_since.elapsed())
    }

    fn maybe_flush_ui_preferences(
        &mut self,
        force: bool,
        reason: &'static str,
    ) -> Result<(), RuntimeError> {
        let Some(dirty_since) = self.ui_preferences_dirty_since else {
            return Ok(());
        };
        if !force && dirty_since.elapsed() < Duration::from_millis(UI_PREFERENCES_WRITE_DEBOUNCE_MS)
        {
            return Ok(());
        }
        self.persist_ui_preferences(reason)
    }

    fn persist_ui_preferences(&mut self, reason: &'static str) -> Result<(), RuntimeError> {
        let preferences = PersistedUiPreferences::from_state(&self.state, self.runtime.theme);
        let started = Instant::now();
        match save_ui_preferences(&self.runtime.ui_preferences_file, &preferences) {
            Ok(bytes_written) => {
                self.ui_preferences_dirty_since = None;
                let elapsed_ms = started.elapsed().as_millis();
                self.logger.log_timing(
                    "ui_preferences_write",
                    &format!(
                        "reason={reason} path={} bytes={} elapsed_ms={elapsed_ms}",
                        self.runtime.ui_preferences_file.display(),
                        bytes_written
                    ),
                )?;
                Ok(())
            }
            Err(error) => {
                self.ui_preferences_dirty_since = None;
                self.logger.info(&format!(
                    "ui_preferences_save_failed path={} error={error}",
                    self.runtime.ui_preferences_file.display()
                ))?;
                self.record_alert(
                    OperatorAlertSource::Diagnostics,
                    OperatorAlertLevel::Warn,
                    format!("UI preferences were not saved: {error}"),
                )
            }
        }
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
        let pending_pull_request_timeout = self
            .pending_pull_request_lookup
            .as_ref()
            .map(|pending| pending.due_at.saturating_duration_since(Instant::now()))
            .unwrap_or(inventory_timeout);
        let pending_extension_timeout = self
            .pending_extension_lookup
            .as_ref()
            .map(|pending| pending.due_at.saturating_duration_since(Instant::now()))
            .unwrap_or(inventory_timeout);
        let pull_request_result_timeout = if self.pull_request_lookup.in_flight_generation.is_some()
        {
            Duration::from_millis(PULL_REQUEST_LOOKUP_RESULT_POLL_MS)
        } else {
            inventory_timeout
        };
        let extension_result_timeout = if self.extension_lookup.in_flight_generation.is_some() {
            Duration::from_millis(EXTENSION_LOOKUP_RESULT_POLL_MS)
        } else {
            inventory_timeout
        };
        let timeout = inventory_timeout
            .min(pending_pull_request_timeout)
            .min(pending_extension_timeout)
            .min(pull_request_result_timeout)
            .min(extension_result_timeout)
            .min(self.ui_preferences_flush_timeout(inventory_timeout));
        if self.state.popup_mode {
            timeout.min(Duration::from_millis(POPUP_INPUT_POLL_MAX_MS))
        } else {
            timeout
        }
    }

    fn schedule_selected_pull_request_refresh(&mut self, previous_workspace: Option<PathBuf>) {
        if self.state.popup_mode {
            self.pending_pull_request_lookup = None;
            return;
        }
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
            self.clear_pending_pull_request_lookup()?;
            return Ok(());
        };

        if selected_workspace != pending.workspace_path {
            self.clear_pending_pull_request_lookup()?;
            self.schedule_selected_pull_request_refresh(Some(pending.workspace_path));
            return Ok(());
        }

        self.pending_pull_request_lookup = None;
        self.maybe_refresh_selected_pull_request(false, "selection-idle")
    }

    fn schedule_selected_extension_refresh(&mut self, previous_workspace: Option<PathBuf>) {
        if self.state.popup_mode {
            self.pending_extension_lookup = None;
            return;
        }
        let current_workspace = self.state.selected_workspace_path();
        if current_workspace == previous_workspace {
            return;
        }

        let Some(workspace_path) = current_workspace else {
            self.pending_extension_lookup = None;
            return;
        };

        if !self.extension_lookup_due(&workspace_path, false) {
            self.pending_extension_lookup = None;
            return;
        }

        self.pending_extension_lookup = Some(PendingExtensionLookup {
            workspace_path,
            due_at: Instant::now() + Duration::from_millis(SELECTION_EXTENSION_LOOKUP_DEBOUNCE_MS),
        });
    }

    fn run_pending_extension_refresh(&mut self) -> Result<(), RuntimeError> {
        let Some(pending) = self.pending_extension_lookup.clone() else {
            return Ok(());
        };

        if pending.due_at > Instant::now() {
            return Ok(());
        }

        let Some(selected_workspace) = self.state.selected_workspace_path() else {
            self.clear_pending_extension_lookup()?;
            return Ok(());
        };

        if selected_workspace != pending.workspace_path {
            self.clear_pending_extension_lookup()?;
            self.schedule_selected_extension_refresh(Some(pending.workspace_path));
            return Ok(());
        }

        self.pending_extension_lookup = None;
        self.maybe_refresh_selected_extensions(false, "selection-idle")
    }

    fn replace_pending_pull_request_lookup(
        &mut self,
        pending: PendingPullRequestLookup,
    ) -> Result<(), RuntimeError> {
        if let Some(previous) = self.pending_pull_request_lookup.replace(pending) {
            self.clear_pull_request_refreshing_for(previous.workspace_path)?;
        }
        Ok(())
    }

    fn clear_pending_pull_request_lookup(&mut self) -> Result<(), RuntimeError> {
        if let Some(pending) = self.pending_pull_request_lookup.take() {
            self.clear_pull_request_refreshing_for(pending.workspace_path)?;
        }
        Ok(())
    }

    fn clear_pull_request_refreshing_for(
        &mut self,
        workspace_path: PathBuf,
    ) -> Result<(), RuntimeError> {
        if self
            .state
            .pull_request_refreshing_workspace
            .as_ref()
            .is_some_and(|refreshing| refreshing == &workspace_path)
        {
            self.apply_runtime_action(Action::SetPullRequestRefreshing {
                workspace_path,
                refreshing: false,
            })?;
        }
        Ok(())
    }

    fn replace_pending_extension_lookup(
        &mut self,
        pending: PendingExtensionLookup,
    ) -> Result<(), RuntimeError> {
        if let Some(previous) = self.pending_extension_lookup.replace(pending) {
            self.clear_extension_refreshing_for(previous.workspace_path)?;
        }
        Ok(())
    }

    fn clear_pending_extension_lookup(&mut self) -> Result<(), RuntimeError> {
        if let Some(pending) = self.pending_extension_lookup.take() {
            self.clear_extension_refreshing_for(pending.workspace_path)?;
        }
        Ok(())
    }

    fn clear_extension_refreshing_for(
        &mut self,
        workspace_path: PathBuf,
    ) -> Result<(), RuntimeError> {
        if self
            .state
            .extension_refreshing_workspace
            .as_ref()
            .is_some_and(|refreshing| refreshing == &workspace_path)
        {
            self.apply_runtime_action(Action::SetExtensionRefreshing {
                workspace_path,
                refreshing: false,
            })?;
        }
        Ok(())
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

    fn extension_lookup_due(&self, workspace_path: &PathBuf, force: bool) -> bool {
        let poll_interval = Duration::from_millis(self.runtime.extension_poll_interval_ms.max(1));
        force
            || !self
                .state
                .extension_cards_cache
                .contains_key(workspace_path)
            || self
                .last_extension_lookup
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

    fn maybe_write_startup_cache(
        &mut self,
        inventory_refresh_complete: bool,
    ) -> Result<(), RuntimeError> {
        if self.state.inventory.sessions.is_empty() {
            return Ok(());
        }
        if self.state.visible_target_count() == 0 {
            return Ok(());
        }
        if self.runtime.source.is_some() {
            return Ok(());
        }
        if matches!(self.runtime.sources.default_scope, SourceScope::All)
            && !inventory_refresh_complete
        {
            return Ok(());
        }

        let now_ms = current_time_ms();
        let write_recently =
            self.startup_cache
                .last_written_at_ms
                .is_some_and(|last_write_at_ms| {
                    now_ms.saturating_sub(last_write_at_ms) < STARTUP_CACHE_WRITE_INTERVAL_MS
                });
        if write_recently {
            return Ok(());
        }

        let write_started = Instant::now();
        match write_startup_cache(&self.runtime, &self.state.inventory) {
            Ok(receipt) => {
                let elapsed_ms = write_started.elapsed().as_millis();
                self.logger.log_timing(
                    "startup_cache_write",
                    &format!(
                        "outcome=written elapsed_ms={elapsed_ms} bytes={} sessions={} panes={} path={}",
                        receipt.bytes_written,
                        self.state.inventory.session_count(),
                        self.state.inventory.pane_count(),
                        receipt.path.display(),
                    ),
                )?;
                self.maybe_log_slow_operation(
                    "startup_cache_write",
                    SLOW_STARTUP_CACHE_WRITE_MS,
                    elapsed_ms,
                    &format!(
                        "outcome=written elapsed_ms={elapsed_ms} bytes={} sessions={} panes={}",
                        receipt.bytes_written,
                        self.state.inventory.session_count(),
                        self.state.inventory.pane_count(),
                    ),
                )?;
                self.startup_cache.last_written_at_ms = Some(receipt.generated_at_ms);
            }
            Err(error) => {
                self.logger
                    .info(&format!("startup_cache_write_failed error={error}"))?;
            }
        }

        Ok(())
    }
}

impl Drop for InventoryRefreshWorker {
    fn drop(&mut self) {
        let _ = self.request_tx.send(InventoryRefreshCommand::Shutdown);
    }
}

impl Drop for PullRequestLookupWorker {
    fn drop(&mut self) {
        let _ = self.request_tx.send(PullRequestLookupCommand::Shutdown);
    }
}

impl Drop for ExtensionLookupWorker {
    fn drop(&mut self) {
        let _ = self.request_tx.send(ExtensionLookupCommand::Shutdown);
    }
}

fn expand_activation_command(template: &str, pane_key: &PaneKey) -> String {
    template
        .replace("{source_id}", &shell_quote(pane_key.source_id.as_str()))
        .replace("{pane_id}", &shell_quote(pane_key.pane_id.as_str()))
}

fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-' | ':' | '='))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
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
            let tmux = TmuxAdapter::new(SystemTmuxBackend::with_target(runtime.tmux_target()));
            while let Ok(command) = request_rx.recv() {
                match command {
                    InventoryRefreshCommand::Refresh(request) => {
                        let refresh_started = Instant::now();
                        if runtime.source.is_some()
                            || runtime.sources.default_scope == SourceScope::All
                        {
                            if runtime
                                .source
                                .as_deref()
                                .is_none_or(|source| source == crate::sources::LOCAL_SOURCE_ID)
                            {
                                let local_stage_started = Instant::now();
                                let local_stage = load_source_local_inventory_refresh_payload(
                                    &tmux,
                                    &runtime,
                                    &request.previous_inventory,
                                    &request.policy,
                                );
                                let _ = result_tx.send(InventoryRefreshResult {
                                    generation: request.generation,
                                    elapsed_ms: local_stage_started.elapsed().as_millis(),
                                    notification_policy_epoch: request.notification_policy_epoch,
                                    complete: false,
                                    result: local_stage,
                                });
                            }
                            let result = load_source_aggregate_inventory_refresh_payload(
                                &tmux,
                                &runtime,
                                &request.previous_inventory,
                                &request.policy,
                            );
                            let _ = result_tx.send(InventoryRefreshResult {
                                generation: request.generation,
                                elapsed_ms: refresh_started.elapsed().as_millis(),
                                notification_policy_epoch: request.notification_policy_epoch,
                                complete: true,
                                result,
                            });
                        } else {
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
                                complete: true,
                                result,
                            });
                        }
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

fn spawn_pull_request_lookup_worker() -> PullRequestLookupWorker {
    let (request_tx, request_rx) = mpsc::channel::<PullRequestLookupCommand>();
    let (result_tx, result_rx) = mpsc::channel::<PullRequestLookupResult>();

    thread::Builder::new()
        .name("foreman-pull-request-lookup".to_string())
        .spawn(move || {
            let pull_requests = PullRequestService::new(SystemPullRequestBackend::new());
            while let Ok(command) = request_rx.recv() {
                match command {
                    PullRequestLookupCommand::Lookup(request) => {
                        let lookup_started = Instant::now();
                        let lookup = match pull_requests.lookup(&request.workspace_path) {
                            Ok(lookup) => lookup,
                            Err(error) => PullRequestLookup::Unavailable {
                                message: error.to_string(),
                            },
                        };
                        let _ = result_tx.send(PullRequestLookupResult {
                            generation: request.generation,
                            workspace_path: request.workspace_path,
                            trigger: request.trigger,
                            elapsed_ms: lookup_started.elapsed().as_millis(),
                            lookup,
                        });
                    }
                    PullRequestLookupCommand::Shutdown => break,
                }
            }
        })
        .expect("pull request lookup worker should spawn");

    PullRequestLookupWorker {
        request_tx,
        result_rx,
        next_generation: 0,
        in_flight_generation: None,
    }
}

fn remote_extension_cards(
    config_file: &std::path::Path,
    pane_key: &PaneKey,
) -> Result<Vec<ControlExtensionCard>, String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let output = Command::new(exe)
        .arg("--config-file")
        .arg(config_file)
        .arg("--source")
        .arg(pane_key.source_id.as_str())
        .arg("extensions")
        .arg("--pane")
        .arg(pane_key.pane_id.as_str())
        .arg("--json")
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())?;
    serde_json::from_value(
        value
            .get("extensionCards")
            .cloned()
            .unwrap_or_else(|| serde_json::Value::Array(Vec::new())),
    )
    .map_err(|error| error.to_string())
}

fn spawn_extension_lookup_worker() -> ExtensionLookupWorker {
    let (request_tx, request_rx) = mpsc::channel::<ExtensionLookupCommand>();
    let (result_tx, result_rx) = mpsc::channel::<ExtensionLookupResult>();

    thread::Builder::new()
        .name("foreman-extension-lookup".to_string())
        .spawn(move || {
            while let Ok(command) = request_rx.recv() {
                match command {
                    ExtensionLookupCommand::Lookup(request) => {
                        let lookup_started = Instant::now();
                        let cards = match request.pane_key.as_ref() {
                            Some(pane_key) if !pane_key.is_local() => {
                                remote_extension_cards(&request.config_file, pane_key)
                                    .unwrap_or_default()
                            }
                            _ => collect_workspace_extensions(std::slice::from_ref(
                                &request.workspace_path,
                            ))
                            .remove(&request.workspace_path)
                            .unwrap_or_default(),
                        };
                        let _ = result_tx.send(ExtensionLookupResult {
                            generation: request.generation,
                            workspace_path: request.workspace_path,
                            trigger: request.trigger,
                            elapsed_ms: lookup_started.elapsed().as_millis(),
                            cards,
                        });
                    }
                    ExtensionLookupCommand::Shutdown => break,
                }
            }
        })
        .expect("extension lookup worker should spawn");

    ExtensionLookupWorker {
        request_tx,
        result_rx,
        next_generation: 0,
        in_flight_generation: None,
    }
}

fn source_cache_file(runtime: &crate::config::RuntimeConfig, source_id: &str) -> PathBuf {
    runtime.startup_cache_dir.join("sources").join(format!(
        "{}.agents.json",
        sanitize_source_cache_key(source_id)
    ))
}

fn sanitize_source_cache_key(source_id: &str) -> String {
    source_id
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => ch,
            _ => '_',
        })
        .collect()
}

fn load_cached_source_inventory(
    runtime: &crate::config::RuntimeConfig,
    descriptor: &SourceDescriptor,
) -> Option<crate::app::Inventory> {
    let path = source_cache_file(runtime, &descriptor.id);
    let bytes = fs::read(path).ok()?;
    let response: AgentsResponse = serde_json::from_slice(&bytes).ok()?;
    if response.schema_version != crate::services::control_api::CONTROL_API_SCHEMA_VERSION {
        return None;
    }
    Some(inventory_from_agents_response(descriptor, response))
}

fn write_cached_source_response(
    runtime: &crate::config::RuntimeConfig,
    descriptor: &SourceDescriptor,
    response: &AgentsResponse,
) {
    let path = source_cache_file(runtime, &descriptor.id);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(bytes) = serde_json::to_vec_pretty(response) {
        let _ = fs::write(path, bytes);
    }
}

fn load_source_local_inventory_refresh_payload(
    tmux: &TmuxAdapter<SystemTmuxBackend>,
    runtime: &crate::config::RuntimeConfig,
    previous_inventory: &crate::app::Inventory,
    policy: &InventoryLoadPolicy,
) -> Result<InventoryRefreshPayload, String> {
    let local_payload = load_inventory_refresh_payload(tmux, runtime, previous_inventory, policy)?;
    let (
        mut local_inventory,
        local_metrics,
        native_timing,
        claude_native,
        codex_native,
        pi_native,
        _source_diagnostics,
    ) = local_payload;
    let local_source = runtime
        .sources
        .get(&crate::sources::SourceId::local())
        .unwrap_or_else(SourceConfig::local);
    let local_descriptor = SourceDescriptor::new(&crate::sources::SourceId::local(), &local_source);
    apply_source_metadata_to_inventory(&mut local_inventory, &local_descriptor);

    if runtime.sources.default_scope == SourceScope::All && runtime.source.is_none() {
        for (source_id, source_config) in runtime.sources.enabled_sources() {
            if source_id.as_str() == crate::sources::LOCAL_SOURCE_ID {
                continue;
            }
            let descriptor = SourceDescriptor::new(&source_id, &source_config);
            if let Some(mut cached_inventory) = load_cached_source_inventory(runtime, &descriptor) {
                local_inventory
                    .sessions
                    .append(&mut cached_inventory.sessions);
            }
        }
    }

    Ok((
        local_inventory,
        local_metrics,
        native_timing,
        claude_native,
        codex_native,
        pi_native,
        Vec::new(),
    ))
}

fn load_source_aggregate_inventory_refresh_payload(
    tmux: &TmuxAdapter<SystemTmuxBackend>,
    runtime: &crate::config::RuntimeConfig,
    previous_inventory: &crate::app::Inventory,
    policy: &InventoryLoadPolicy,
) -> Result<InventoryRefreshPayload, String> {
    let local_started = Instant::now();
    let local_payload = load_inventory_refresh_payload(tmux, runtime, previous_inventory, policy)?;
    let (
        mut local_inventory,
        local_metrics,
        native_timing,
        claude_native,
        codex_native,
        pi_native,
        _source_diagnostics,
    ) = local_payload;
    let selected_source = runtime.source.as_deref();
    let local_source = runtime
        .sources
        .get(&crate::sources::SourceId::local())
        .unwrap_or_else(SourceConfig::local);
    let local_descriptor = SourceDescriptor::new(&crate::sources::SourceId::local(), &local_source);
    apply_source_metadata_to_inventory(&mut local_inventory, &local_descriptor);

    let mut aggregate_inventory =
        if selected_source.is_none_or(|source| source == crate::sources::LOCAL_SOURCE_ID) {
            local_inventory
        } else {
            crate::app::Inventory::default()
        };
    let mut diagnostics: Vec<SourceDiagnostic> = Vec::new();
    let remote_handles = runtime
        .sources
        .enabled_sources()
        .into_iter()
        .filter(|(source_id, _)| source_id.as_str() != crate::sources::LOCAL_SOURCE_ID)
        .filter(|(source_id, _)| {
            selected_source.is_none_or(|selected| selected == source_id.as_str())
        })
        .filter_map(|(source_id, source_config)| match source_config.clone() {
            SourceConfig::Ssh { .. } => {
                let descriptor = SourceDescriptor::new(&source_id, &source_config);
                let timeout_ms = runtime.sources.query_timeout_ms;
                let cache_runtime = runtime.clone();
                Some(thread::spawn(move || {
                    let source = SshSource::new(source_id, source_config, timeout_ms);
                    let started = Instant::now();
                    match source.agents() {
                        Ok(response) => {
                            write_cached_source_response(&cache_runtime, &descriptor, &response);
                            let source_diagnostics = response.source_diagnostics.clone();
                            Ok((
                                inventory_from_agents_response(&descriptor, response),
                                source_diagnostics,
                            ))
                        }
                        Err(error) => {
                            let diagnostic = SourceDiagnostic::warning(
                                &descriptor,
                                error.code,
                                error.message,
                                error.retryable,
                                Some(started.elapsed().as_millis() as u64),
                            );
                            Err((descriptor, diagnostic))
                        }
                    }
                }))
            }
            SourceConfig::Snapshot { .. } => {
                let descriptor = SourceDescriptor::new(&source_id, &source_config);
                Some(thread::spawn(move || {
                    let source = SnapshotSource::new(source_id, source_config);
                    let started = Instant::now();
                    match source.agents() {
                        Ok(response) => {
                            let source_diagnostics = response.source_diagnostics.clone();
                            Ok((
                                inventory_from_agents_response(&descriptor, response),
                                source_diagnostics,
                            ))
                        }
                        Err(error) => {
                            let diagnostic = SourceDiagnostic::warning(
                                &descriptor,
                                error.code,
                                error.message,
                                error.retryable,
                                Some(started.elapsed().as_millis() as u64),
                            );
                            Err((descriptor, diagnostic))
                        }
                    }
                }))
            }
            SourceConfig::Local { .. } => None,
        })
        .collect::<Vec<_>>();
    for handle in remote_handles {
        match handle.join() {
            Ok(Ok((mut inventory, mut source_diagnostics))) => {
                aggregate_inventory.sessions.append(&mut inventory.sessions);
                diagnostics.append(&mut source_diagnostics);
            }
            Ok(Err((descriptor, diagnostic))) => {
                if let Some(mut cached_inventory) =
                    load_cached_source_inventory(runtime, &descriptor)
                {
                    aggregate_inventory
                        .sessions
                        .append(&mut cached_inventory.sessions);
                }
                diagnostics.push(diagnostic);
            }
            Err(_) => diagnostics.push(SourceDiagnostic {
                level: "warning".to_string(),
                code: "source.thread.panicked".to_string(),
                source_id: "unknown".to_string(),
                source_label: "unknown".to_string(),
                source_kind: "ssh".to_string(),
                message: "source refresh worker panicked".to_string(),
                retryable: true,
                duration_ms: None,
                last_success_unix_ms: None,
            }),
        }
    }

    let _ = local_started;

    Ok((
        aggregate_inventory,
        local_metrics,
        native_timing,
        claude_native,
        codex_native,
        pi_native,
        diagnostics,
    ))
}

fn apply_source_metadata_to_inventory(
    inventory: &mut crate::app::Inventory,
    descriptor: &SourceDescriptor,
) {
    let source = crate::app::SourceMetadata::with_display(
        descriptor.id.clone(),
        descriptor.display_label.clone(),
        descriptor.kind.clone(),
        descriptor.show_label,
    );
    for session in &mut inventory.sessions {
        session.source = source.clone();
        for window in &mut session.windows {
            window.source = source.clone();
            for pane in &mut window.panes {
                pane.source = source.clone();
            }
        }
    }
}

fn inventory_from_agents_response(
    descriptor: &SourceDescriptor,
    response: AgentsResponse,
) -> crate::app::Inventory {
    let source = crate::app::SourceMetadata::with_display(
        descriptor.id.clone(),
        descriptor.display_label.clone(),
        descriptor.kind.clone(),
        descriptor.show_label,
    );
    let mut sessions: Vec<crate::app::Session> = Vec::new();
    for entry in response.entries {
        let pane = pane_from_agent_entry(&source, &entry);
        let session_id = crate::app::SessionId::new(entry.session_id.clone());
        let window_id = crate::app::WindowId::new(entry.window_id.clone());
        if let Some(session) = sessions
            .iter_mut()
            .find(|session| session.id == session_id && session.source.id == source.id)
        {
            if let Some(window) = session
                .windows
                .iter_mut()
                .find(|window| window.id == window_id && window.source.id == source.id)
            {
                window.panes.push(pane);
            } else {
                session.windows.push(crate::app::Window {
                    id: window_id,
                    source: source.clone(),
                    name: entry.window_name,
                    panes: vec![pane],
                });
            }
        } else {
            sessions.push(crate::app::Session {
                id: session_id,
                source: source.clone(),
                name: entry.session_name,
                windows: vec![crate::app::Window {
                    id: window_id,
                    source: source.clone(),
                    name: entry.window_name,
                    panes: vec![pane],
                }],
            });
        }
    }
    crate::app::Inventory::new(sessions)
}

fn pane_from_agent_entry(
    source: &crate::app::SourceMetadata,
    entry: &AgentEntry,
) -> crate::app::Pane {
    let agent = parse_harness(entry.harness.as_deref()).map(|harness| crate::app::AgentSnapshot {
        harness,
        status: parse_status(&entry.status),
        observed_status: parse_status(&entry.status),
        integration_mode: parse_integration_mode(entry.integration_mode.as_deref()),
        activity_score: entry.activity_score,
        debounce_ticks: 0,
        active_run_count: entry.active_run_count,
        last_status_change_unix_millis: entry.last_status_change_unix_ms,
    });
    crate::app::Pane {
        id: crate::app::PaneId::new(entry.pane_id.clone()),
        source: source.clone(),
        title: entry.title.clone(),
        current_command: entry.current_command.clone(),
        runtime_command: entry.runtime_command.clone(),
        working_dir: entry.working_dir.as_ref().map(PathBuf::from),
        activity_unix_millis: entry.last_activity_unix_ms,
        preview: entry.preview.clone(),
        preview_provenance: parse_preview_provenance(&entry.preview_provenance),
        agent,
    }
}

fn parse_harness(value: Option<&str>) -> Option<crate::app::HarnessKind> {
    match value? {
        "claude-code" => Some(crate::app::HarnessKind::ClaudeCode),
        "codex-cli" => Some(crate::app::HarnessKind::CodexCli),
        "pi" => Some(crate::app::HarnessKind::Pi),
        "gemini-cli" => Some(crate::app::HarnessKind::GeminiCli),
        "opencode" => Some(crate::app::HarnessKind::OpenCode),
        _ => None,
    }
}

fn parse_status(value: &str) -> crate::app::AgentStatus {
    match value {
        "working" => crate::app::AgentStatus::Working,
        "needs-attention" => crate::app::AgentStatus::NeedsAttention,
        "idle" => crate::app::AgentStatus::Idle,
        "error" => crate::app::AgentStatus::Error,
        _ => crate::app::AgentStatus::Unknown,
    }
}

fn parse_integration_mode(value: Option<&str>) -> crate::app::IntegrationMode {
    match value {
        Some("native") => crate::app::IntegrationMode::Native,
        _ => crate::app::IntegrationMode::Compatibility,
    }
}

fn parse_preview_provenance(value: &str) -> crate::app::PreviewProvenance {
    match value {
        "pending-capture" => crate::app::PreviewProvenance::PendingCapture,
        "reused-cached" => crate::app::PreviewProvenance::ReusedCached,
        "capture-failed" => crate::app::PreviewProvenance::CaptureFailed,
        _ => crate::app::PreviewProvenance::Captured,
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
        Vec::new(),
    ))
}

fn should_persist_ui_preferences(action_label: &str) -> bool {
    matches!(
        action_label,
        "move-selection"
            | "set-selection"
            | "commit-search-selection"
            | "toggle-show-non-agent-sessions"
            | "toggle-show-non-agent-panes"
            | "cycle-harness-filter"
            | "toggle-session-collapsed"
            | "set-sort-mode"
            | "cycle-theme"
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{
        inventory, HarnessKind, PaneBuilder, SelectionTarget, SessionBuilder, WindowBuilder,
    };
    use crate::cli::{Cli, PreparedBootstrap};
    use crate::config::{resolve_paths, AppConfig, RuntimeConfig};
    use crate::integrations::{
        ClaudeNativeOverlaySummary, CodexNativeOverlaySummary, PiNativeOverlaySummary,
    };
    use crate::services::logging::RunLogger;
    use clap::Parser;
    use tempfile::tempdir;

    #[test]
    fn abandoned_pending_pull_request_refresh_clears_refreshing_marker() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config_file = temp_dir.path().join("config.toml");
        let log_dir = temp_dir.path().join("logs");
        let paths = resolve_paths(Some(&config_file), Some(&log_dir)).expect("paths resolve");
        let cli = Cli::parse_from([
            "foreman",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
        ]);
        let runtime_config = RuntimeConfig::from_sources(paths, AppConfig::default(), &cli);
        let alpha_workspace = temp_dir.path().join("alpha");
        let beta_workspace = temp_dir.path().join("beta");
        let inventory = inventory([
            SessionBuilder::new("alpha").window(
                WindowBuilder::new("alpha:agents").pane(
                    PaneBuilder::agent("alpha:pane", HarnessKind::ClaudeCode)
                        .working_dir(alpha_workspace.to_string_lossy().as_ref()),
                ),
            ),
            SessionBuilder::new("beta").window(
                WindowBuilder::new("beta:agents").pane(
                    PaneBuilder::agent("beta:pane", HarnessKind::CodexCli)
                        .working_dir(beta_workspace.to_string_lossy().as_ref()),
                ),
            ),
        ]);
        let mut state = AppState::with_inventory(inventory);
        state.selection = Some(SelectionTarget::Pane("beta:pane".into()));
        state.pull_request_refreshing_workspace = Some(alpha_workspace.clone());
        let logger = RunLogger::start(
            &runtime_config.log_dir,
            runtime_config.log_retention,
            runtime_config.log_verbosity,
        )
        .expect("logger should start");
        let mut runtime = DashboardRuntime::new(PreparedBootstrap {
            runtime: runtime_config,
            logger,
            state,
            claude_native: ClaudeNativeOverlaySummary::default(),
            codex_native: CodexNativeOverlaySummary::default(),
            pi_native: PiNativeOverlaySummary::default(),
            startup_cache_generated_at_ms: None,
            pending_selection_restore: None,
            persistent_runtime_diagnostics: Vec::new(),
        });
        runtime.pending_pull_request_lookup = Some(PendingPullRequestLookup {
            workspace_path: alpha_workspace.clone(),
            due_at: Instant::now() - Duration::from_millis(1),
        });

        runtime
            .run_pending_pull_request_refresh()
            .expect("pending refresh should run");

        assert_ne!(
            runtime.state.pull_request_refreshing_workspace,
            Some(alpha_workspace)
        );
        assert_eq!(
            runtime
                .pending_pull_request_lookup
                .as_ref()
                .map(|pending| pending.workspace_path.clone()),
            Some(beta_workspace)
        );
    }

    #[test]
    fn remote_agent_response_becomes_source_scoped_inventory() {
        let descriptor = SourceDescriptor {
            id: "coder".to_string(),
            label: "Coder".to_string(),
            kind: "ssh".to_string(),
            enabled: true,
            display_label: "Coder".to_string(),
            show_label: true,
        };
        let mut entry = AgentEntry::test_entry("%42");
        entry.source_id = "coder".to_string();
        entry.source_label = "Coder".to_string();
        entry.source_kind = "ssh".to_string();
        entry.session_id = "$1".to_string();
        entry.session_name = "dots".to_string();
        entry.window_id = "@1".to_string();
        entry.window_name = "agents".to_string();
        entry.harness = Some("pi".to_string());
        entry.status = "working".to_string();
        entry.preview = "remote output".to_string();
        let response = AgentsResponse {
            schema_version: crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
            generated_at_unix_ms: 1,
            inventory: crate::services::control_api::ControlInventorySummary {
                total_sessions: 1,
                total_windows: 1,
                total_panes: 1,
                visible_sessions: 1,
                visible_windows: 1,
                visible_panes: 1,
            },
            entries: vec![entry],
            diagnostics: Vec::new(),
            sources: Vec::new(),
            source_diagnostics: Vec::new(),
            partial_failure_count: 0,
        };

        let inventory = inventory_from_agents_response(&descriptor, response);
        let pane_key = crate::app::PaneKey::new(crate::app::SourceId::new("coder"), "%42".into());
        let pane = inventory.pane(&pane_key).expect("remote pane");

        assert_eq!(pane.source.label, "Coder");
        assert_eq!(pane.id.as_str(), "%42");
        assert_eq!(pane.preview, "remote output");
        assert_eq!(pane.agent.as_ref().unwrap().harness, HarnessKind::Pi);
        assert_eq!(
            pane.agent.as_ref().unwrap().status,
            crate::app::AgentStatus::Working
        );
        assert_eq!(
            inventory.visible_targets(
                &crate::app::Filters::default(),
                &Default::default(),
                Default::default()
            ),
            vec![
                SelectionTarget::Session(crate::app::SessionKey::new(
                    crate::app::SourceId::new("coder"),
                    "$1".into()
                )),
                SelectionTarget::Pane(pane_key),
            ]
        );
    }
}
