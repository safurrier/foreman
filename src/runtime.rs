use crate::adapters::tmux::{SystemTmuxBackend, TmuxAdapter};
use crate::app::{
    action_for_command, map_key_event, reduce, Action, AppState, DraftEdit, Effect, OperatorAlert,
    OperatorAlertLevel, OperatorAlertSource,
};
use crate::cli::PreparedBootstrap;
use crate::integrations::{apply_configured_claude_signals, ClaudeNativeOverlaySummary};
use crate::services::logging::RunLogger;
use crate::services::notifications::{
    build_notification_dispatcher, NotificationDispatcher, NotificationRequest,
};
use crate::services::pull_requests::{
    PullRequestLookup, PullRequestService, SystemPullRequestBackend,
};
use crate::services::system_stats::{SysinfoSystemStatsBackend, SystemStatsService};
use crate::ui::render::render;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::collections::BTreeMap;
use std::fmt;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub(crate) fn run(prepared: PreparedBootstrap) -> Result<(), RuntimeError> {
    let _terminal_guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut runtime = DashboardRuntime::new(prepared);
    runtime.maybe_refresh_selected_pull_request(true)?;
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
}

impl DashboardRuntime {
    fn new(prepared: PreparedBootstrap) -> Self {
        let tmux = TmuxAdapter::new(SystemTmuxBackend::new(prepared.runtime.tmux_socket.clone()));

        Self {
            tmux,
            pull_requests: PullRequestService::new(SystemPullRequestBackend::new()),
            notifications: build_notification_dispatcher(&prepared.runtime.notification_backends),
            system_stats: SystemStatsService::new(SysinfoSystemStatsBackend::new()),
            last_pull_request_lookup: BTreeMap::new(),
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

        self.logger.info("dashboard_started")?;
        self.logger.debug("dashboard_event_loop_started")?;

        loop {
            terminal.draw(|frame| render(frame, &self.state))?;

            let timeout = poll_interval.saturating_sub(last_inventory_refresh.elapsed());
            if event::poll(timeout)? {
                match event::read()? {
                    Event::Key(key) => {
                        if let Some(command) = map_key_event(key, self.state.focus, self.state.mode)
                        {
                            let action = action_for_command(&self.state, command);
                            if self.apply_action(action)? {
                                break;
                            }
                            self.maybe_refresh_selected_pull_request(false)?;
                        }
                    }
                    Event::Paste(text) => {
                        self.apply_paste(text)?;
                        self.maybe_refresh_selected_pull_request(false)?;
                    }
                    Event::Resize(_, _)
                    | Event::Mouse(_)
                    | Event::FocusGained
                    | Event::FocusLost => {}
                }
            }

            if last_inventory_refresh.elapsed() >= poll_interval {
                self.refresh_inventory()?;
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
        let effects = reduce(&mut self.state, action);
        self.execute_effects(effects)
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
                            self.refresh_inventory()?;
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
                        self.refresh_inventory()?;
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
                        self.refresh_inventory()?;
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
                Effect::Quit => {
                    should_quit = true;
                }
            }
        }

        Ok(should_quit)
    }

    fn refresh_inventory(&mut self) -> Result<(), RuntimeError> {
        self.logger.debug("inventory_refresh_started")?;
        match self.load_inventory_with_native() {
            Ok((inventory, claude_native)) => {
                self.apply_runtime_action(Action::SetStartupError(None))?;
                self.clear_alert_source(OperatorAlertSource::Tmux)?;
                self.apply_runtime_action(Action::ReplaceInventory(inventory))?;

                let summary = self.state.inventory_summary();
                self.logger.log_inventory(&summary)?;
                self.logger.log_claude_native_summary(&claude_native)?;
                for warning in &claude_native.warnings {
                    self.logger.log_claude_native_warning(warning)?;
                }
            }
            Err(error_message) => {
                self.apply_runtime_action(Action::SetStartupError(Some(error_message.clone())))?;
                self.logger.log_tmux_error(&error_message)?;
                self.record_alert(
                    OperatorAlertSource::Tmux,
                    OperatorAlertLevel::Warn,
                    format!("tmux unavailable: {error_message}"),
                )?;
            }
        }

        self.refresh_system_stats()?;
        self.maybe_refresh_selected_pull_request(false)?;
        Ok(())
    }

    fn load_inventory_with_native(
        &self,
    ) -> Result<(crate::app::Inventory, ClaudeNativeOverlaySummary), String> {
        let mut inventory = self
            .tmux
            .load_inventory(self.runtime.capture_lines)
            .map_err(|error| error.to_string())?;

        let claude_native = apply_configured_claude_signals(
            &mut inventory,
            self.runtime.claude_native_dir.as_deref(),
            self.runtime.claude_integration_preference,
        );

        Ok((inventory, claude_native))
    }

    fn refresh_system_stats(&mut self) -> Result<(), RuntimeError> {
        let snapshot = self.system_stats.snapshot().unwrap_or_default();
        self.logger.log_system_stats(&snapshot)?;
        self.apply_runtime_action(Action::SetSystemStats(snapshot))?;
        Ok(())
    }

    fn maybe_refresh_selected_pull_request(&mut self, force: bool) -> Result<(), RuntimeError> {
        if !self.runtime.pull_request_monitoring_enabled {
            return Ok(());
        }

        let Some(workspace_path) = self.state.selected_workspace_path() else {
            return Ok(());
        };

        let poll_interval =
            Duration::from_millis(self.runtime.pull_request_poll_interval_ms.max(1));
        let lookup_due = force
            || !self.state.pull_request_cache.contains_key(&workspace_path)
            || self
                .last_pull_request_lookup
                .get(&workspace_path)
                .is_none_or(|last_lookup| last_lookup.elapsed() >= poll_interval);
        if !lookup_due {
            return Ok(());
        }

        let lookup = match self.pull_requests.lookup(&workspace_path) {
            Ok(lookup) => lookup,
            Err(error) => PullRequestLookup::Unavailable {
                message: error.to_string(),
            },
        };

        self.last_pull_request_lookup
            .insert(workspace_path.clone(), Instant::now());
        self.logger
            .log_pull_request_lookup(&workspace_path, &lookup)?;
        self.apply_runtime_action(Action::SetPullRequestLookup {
            workspace_path,
            lookup,
        })?;
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
