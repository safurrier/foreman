use crate::app::{InventorySummary, OperatorAlert};
use crate::config::{LogVerbosity, RuntimeConfig};
use crate::integrations::{
    ClaudeNativeOverlaySummary, CodexNativeOverlaySummary, PiNativeOverlaySummary,
};
use crate::services::notifications::{
    NotificationDecision, NotificationDispatchReceipt, NotificationError, NotificationRequest,
};
use crate::services::pull_requests::PullRequestLookup;
use crate::services::system_stats::SystemStatsSnapshot;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const RUN_LOG_PREFIX: &str = "foreman-";
const RUN_LOG_SUFFIX: &str = ".log";
const LATEST_LOG_NAME: &str = "latest.log";

#[derive(Debug)]
pub struct RunLogger {
    run_path: PathBuf,
    latest_path: PathBuf,
    verbosity: LogVerbosity,
    run_file: File,
    latest_file: File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunLogSummary {
    pub run_path: PathBuf,
    pub latest_path: PathBuf,
}

impl RunLogger {
    pub fn start(
        log_dir: &Path,
        retain_run_logs: usize,
        verbosity: LogVerbosity,
    ) -> io::Result<Self> {
        fs::create_dir_all(log_dir)?;

        let run_id = current_run_id();
        let run_path = log_dir.join(format!("{RUN_LOG_PREFIX}{run_id}{RUN_LOG_SUFFIX}"));
        let latest_path = log_dir.join(LATEST_LOG_NAME);

        let run_file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&run_path)?;
        let latest_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&latest_path)?;

        let mut logger = Self {
            run_path,
            latest_path,
            verbosity,
            run_file,
            latest_file,
        };
        logger.info("run_started")?;
        cleanup_old_logs(log_dir, retain_run_logs, &logger.run_path)?;
        Ok(logger)
    }

    pub fn info(&mut self, message: &str) -> io::Result<()> {
        self.write_line("INFO", message)
    }

    pub fn debug(&mut self, message: &str) -> io::Result<()> {
        if self.verbosity.includes_debug() {
            self.write_line("DEBUG", message)
        } else {
            Ok(())
        }
    }

    pub fn log_bootstrap(&mut self, runtime: &RuntimeConfig) -> io::Result<()> {
        self.write_line(
            "INFO",
            &format!(
                "bootstrap_complete config={} poll_interval_ms={} capture_lines={} popup={} pr_monitoring_enabled={} pr_poll_interval_ms={} notifications_enabled={} notification_cooldown_ticks={} notification_profile={} notification_backends={} claude_integration_preference={} codex_integration_preference={} pi_integration_preference={} log_verbosity={} tmux_socket={}",
                runtime.config_file.display(),
                runtime.poll_interval_ms,
                runtime.capture_lines,
                runtime.popup,
                runtime.pull_request_monitoring_enabled,
                runtime.pull_request_poll_interval_ms,
                runtime.notifications_enabled,
                runtime.notification_cooldown_ticks,
                runtime.notification_profile.label(),
                runtime
                    .notification_backends
                    .iter()
                    .map(|backend| backend.label())
                    .collect::<Vec<_>>()
                    .join(","),
                runtime.claude_integration_preference.label(),
                runtime.codex_integration_preference.label(),
                runtime.pi_integration_preference.label(),
                runtime.log_verbosity.label(),
                runtime
                    .tmux_socket
                    .as_deref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "default".to_string())
            ),
        )
    }

    pub fn log_inventory(&mut self, summary: &InventorySummary) -> io::Result<()> {
        self.write_line(
            "INFO",
            &format!(
                "inventory_loaded sessions={} windows={} panes={} visible_sessions={} visible_windows={} visible_panes={} startup_error={}",
                summary.total_sessions,
                summary.total_windows,
                summary.total_panes,
                summary.visible_sessions,
                summary.visible_windows,
                summary.visible_panes,
                summary.startup_error.is_some()
            ),
        )
    }

    pub fn log_system_stats(&mut self, snapshot: &SystemStatsSnapshot) -> io::Result<()> {
        self.write_line(
            "INFO",
            &format!(
                "system_stats_snapshot cpu_pressure={} memory_pressure={}",
                snapshot
                    .cpu_pressure_percent
                    .map(|value| format!("{value}%"))
                    .unwrap_or_else(|| "?".to_string()),
                snapshot
                    .memory_pressure_percent
                    .map(|value| format!("{value}%"))
                    .unwrap_or_else(|| "?".to_string())
            ),
        )
    }

    pub fn log_tmux_error(&mut self, error: &str) -> io::Result<()> {
        self.write_line("WARN", &format!("tmux_bootstrap_error {error}"))
    }

    pub fn log_pull_request_lookup(
        &mut self,
        workspace_path: &Path,
        lookup: &PullRequestLookup,
    ) -> io::Result<()> {
        let outcome = match lookup {
            PullRequestLookup::Unknown => "unknown".to_string(),
            PullRequestLookup::Missing => "missing".to_string(),
            PullRequestLookup::Unavailable { message } => format!("unavailable:{message}"),
            PullRequestLookup::Available(pull_request) => {
                format!(
                    "available:#{}:{}",
                    pull_request.number,
                    pull_request.status.label()
                )
            }
        };
        self.write_line(
            "INFO",
            &format!(
                "pull_request_lookup workspace={} outcome={}",
                workspace_path.display(),
                outcome
            ),
        )
    }

    pub fn log_operator_alert(&mut self, alert: &OperatorAlert) -> io::Result<()> {
        self.write_line(
            alert.level.label(),
            &format!(
                "operator_alert source={} level={} message={}",
                alert.source.label(),
                alert.level.label().to_ascii_lowercase(),
                alert.message
            ),
        )
    }

    pub fn log_claude_native_summary(
        &mut self,
        summary: &ClaudeNativeOverlaySummary,
    ) -> io::Result<()> {
        self.write_line(
            "INFO",
            &format!(
                "claude_native_summary applied={} fallback_to_compatibility={} warnings={}",
                summary.applied,
                summary.fallback_to_compatibility,
                summary.warnings.len()
            ),
        )
    }

    pub fn log_claude_native_warning(&mut self, warning: &str) -> io::Result<()> {
        self.write_line("WARN", &format!("claude_native_warning {warning}"))
    }

    pub fn log_codex_native_summary(
        &mut self,
        summary: &CodexNativeOverlaySummary,
    ) -> io::Result<()> {
        self.write_line(
            "INFO",
            &format!(
                "codex_native_summary applied={} fallback_to_compatibility={} warnings={}",
                summary.applied,
                summary.fallback_to_compatibility,
                summary.warnings.len()
            ),
        )
    }

    pub fn log_codex_native_warning(&mut self, warning: &str) -> io::Result<()> {
        self.write_line("WARN", &format!("codex_native_warning {warning}"))
    }

    pub fn log_pi_native_summary(&mut self, summary: &PiNativeOverlaySummary) -> io::Result<()> {
        self.write_line(
            "INFO",
            &format!(
                "pi_native_summary applied={} fallback_to_compatibility={} warnings={}",
                summary.applied,
                summary.fallback_to_compatibility,
                summary.warnings.len()
            ),
        )
    }

    pub fn log_pi_native_warning(&mut self, warning: &str) -> io::Result<()> {
        self.write_line("WARN", &format!("pi_native_warning {warning}"))
    }

    pub fn log_notification_decision(&mut self, decision: &NotificationDecision) -> io::Result<()> {
        let action = if decision.request.is_some() {
            "emit"
        } else {
            "suppress"
        };
        self.write_line(
            "INFO",
            &format!(
                "notification_decision pane_id={} kind={} action={} reason={}",
                decision.pane_id.as_str(),
                decision.kind.label(),
                action,
                decision.reason.label()
            ),
        )
    }

    pub fn log_notification_backend_selected(
        &mut self,
        request: &NotificationRequest,
        receipt: &NotificationDispatchReceipt,
    ) -> io::Result<()> {
        self.write_line(
            "INFO",
            &format!(
                "notification_backend_selected pane_id={} kind={} backend={}",
                request.pane_id.as_str(),
                request.kind.label(),
                receipt.backend_name
            ),
        )
    }

    pub fn log_notification_backend_failure(
        &mut self,
        request: &NotificationRequest,
        error: &NotificationError,
    ) -> io::Result<()> {
        self.write_line(
            "WARN",
            &format!(
                "notification_backend_failure pane_id={} kind={} error={}",
                request.pane_id.as_str(),
                request.kind.label(),
                error
            ),
        )
    }

    pub fn summary(&self) -> RunLogSummary {
        RunLogSummary {
            run_path: self.run_path.clone(),
            latest_path: self.latest_path.clone(),
        }
    }

    fn write_line(&mut self, level: &str, message: &str) -> io::Result<()> {
        let line = format!("[{level}] {message}\n");
        self.run_file.write_all(line.as_bytes())?;
        self.latest_file.write_all(line.as_bytes())?;
        self.run_file.flush()?;
        self.latest_file.flush()?;
        Ok(())
    }
}

fn cleanup_old_logs(log_dir: &Path, retain_run_logs: usize, current_run: &Path) -> io::Result<()> {
    let mut run_logs: Vec<PathBuf> = fs::read_dir(log_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path != current_run)
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .map(|name| name.starts_with(RUN_LOG_PREFIX) && name.ends_with(RUN_LOG_SUFFIX))
                .unwrap_or(false)
        })
        .collect();

    run_logs.sort();
    if run_logs.len() < retain_run_logs {
        return Ok(());
    }

    let remove_count = run_logs
        .len()
        .saturating_sub(retain_run_logs.saturating_sub(1));
    for old_log in run_logs.into_iter().take(remove_count) {
        fs::remove_file(old_log)?;
    }

    Ok(())
}

fn current_run_id() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch");
    format!("{:020}", duration.as_millis())
}

#[cfg(test)]
mod tests {
    use super::RunLogger;
    use crate::app::{
        InventorySummary, NotificationKind, OperatorAlert, OperatorAlertLevel, OperatorAlertSource,
    };
    use crate::config::{
        IntegrationPreference, LogVerbosity, NotificationBackendName, RuntimeConfig,
    };
    use crate::integrations::{
        ClaudeNativeOverlaySummary, CodexNativeOverlaySummary, PiNativeOverlaySummary,
    };
    use crate::services::notifications::{
        NotificationDecision, NotificationDecisionReason, NotificationDispatchReceipt,
        NotificationError, NotificationRequest,
    };
    use crate::services::pull_requests::{PullRequestData, PullRequestLookup, PullRequestStatus};
    use crate::services::system_stats::SystemStatsSnapshot;
    use std::path::Path;
    use tempfile::tempdir;

    fn runtime_config(log_dir: &Path) -> RuntimeConfig {
        RuntimeConfig {
            config_file: Path::new("/tmp/config.toml").to_path_buf(),
            log_dir: log_dir.to_path_buf(),
            tmux_socket: None,
            claude_native_dir: None,
            codex_native_dir: None,
            pi_native_dir: None,
            log_verbosity: LogVerbosity::Info,
            poll_interval_ms: 1_000,
            capture_lines: 200,
            popup: false,
            pull_request_monitoring_enabled: true,
            pull_request_poll_interval_ms: 30_000,
            notifications_enabled: true,
            notification_cooldown_ticks: 3,
            notification_backends: vec![
                NotificationBackendName::NotifySend,
                NotificationBackendName::OsaScript,
            ],
            notification_profile: crate::app::NotificationProfile::All,
            claude_integration_preference: IntegrationPreference::Auto,
            codex_integration_preference: IntegrationPreference::Auto,
            pi_integration_preference: IntegrationPreference::Auto,
            log_retention: 2,
        }
    }

    #[test]
    fn start_creates_run_log_and_latest_log() {
        let temp_dir = tempdir().expect("temp dir should exist");

        let mut logger =
            RunLogger::start(temp_dir.path(), 5, LogVerbosity::Info).expect("logger should start");
        logger.info("hello").expect("log write should succeed");

        let summary = logger.summary();
        assert!(summary.run_path.exists());
        assert!(summary.latest_path.exists());

        let latest_contents =
            std::fs::read_to_string(summary.latest_path).expect("latest log should be readable");
        assert!(latest_contents.contains("hello"));
    }

    #[test]
    fn start_removes_old_logs_past_retention_limit() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(
            temp_dir.path().join("foreman-00000000000000000001.log"),
            "old",
        )
        .expect("old log should be created");
        std::fs::write(
            temp_dir.path().join("foreman-00000000000000000002.log"),
            "older",
        )
        .expect("old log should be created");
        std::fs::write(
            temp_dir.path().join("foreman-00000000000000000003.log"),
            "newest",
        )
        .expect("old log should be created");

        let _logger =
            RunLogger::start(temp_dir.path(), 2, LogVerbosity::Info).expect("logger should start");

        let mut run_logs: Vec<_> = std::fs::read_dir(temp_dir.path())
            .expect("log dir should be readable")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name())
            .map(|value| value.to_string_lossy().into_owned())
            .filter(|name| name.starts_with("foreman-") && name.ends_with(".log"))
            .collect();
        run_logs.sort();

        assert_eq!(run_logs.len(), 2);
    }

    #[test]
    fn bootstrap_log_writes_runtime_summary() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let runtime = runtime_config(temp_dir.path());
        let mut logger =
            RunLogger::start(temp_dir.path(), 2, LogVerbosity::Info).expect("logger should start");

        logger
            .log_bootstrap(&runtime)
            .expect("bootstrap log should succeed");

        let contents =
            std::fs::read_to_string(logger.summary().run_path).expect("run log should be readable");
        assert!(contents.contains("bootstrap_complete"));
        assert!(contents.contains("poll_interval_ms=1000"));
        assert!(contents.contains("pr_monitoring_enabled=true"));
        assert!(contents.contains("pr_poll_interval_ms=30000"));
        assert!(contents.contains("notification_cooldown_ticks=3"));
        assert!(contents.contains("notification_profile=ALL"));
        assert!(contents.contains("notification_backends=notify-send,osascript"));
        assert!(contents.contains("claude_integration_preference=auto"));
        assert!(contents.contains("codex_integration_preference=auto"));
        assert!(contents.contains("pi_integration_preference=auto"));
        assert!(contents.contains("log_verbosity=info"));
    }

    #[test]
    fn debug_log_lines_only_write_when_enabled() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut info_logger =
            RunLogger::start(temp_dir.path(), 5, LogVerbosity::Info).expect("logger should start");
        info_logger
            .debug("debug_only_message")
            .expect("debug write should succeed");

        let info_contents = std::fs::read_to_string(info_logger.summary().run_path)
            .expect("run log should be readable");
        assert!(!info_contents.contains("debug_only_message"));

        let debug_dir = tempdir().expect("temp dir should exist");
        let mut debug_logger = RunLogger::start(debug_dir.path(), 5, LogVerbosity::Debug)
            .expect("logger should start");
        debug_logger
            .debug("debug_only_message")
            .expect("debug write should succeed");

        let debug_contents = std::fs::read_to_string(debug_logger.summary().run_path)
            .expect("run log should be readable");
        assert!(debug_contents.contains("[DEBUG] debug_only_message"));
    }

    #[test]
    fn inventory_log_writes_visible_counts() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut logger =
            RunLogger::start(temp_dir.path(), 2, LogVerbosity::Info).expect("logger should start");

        logger
            .log_inventory(&InventorySummary {
                total_sessions: 3,
                total_windows: 3,
                total_panes: 4,
                visible_sessions: 2,
                visible_windows: 2,
                visible_panes: 2,
                startup_error: None,
            })
            .expect("inventory log should succeed");

        let contents =
            std::fs::read_to_string(logger.summary().run_path).expect("run log should be readable");
        assert!(contents.contains("inventory_loaded"));
        assert!(contents.contains("visible_sessions=2"));
        assert!(contents.contains("visible_panes=2"));
    }

    #[test]
    fn system_stats_log_writes_header_snapshot() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut logger =
            RunLogger::start(temp_dir.path(), 2, LogVerbosity::Info).expect("logger should start");

        logger
            .log_system_stats(&SystemStatsSnapshot {
                cpu_pressure_percent: Some(21),
                memory_pressure_percent: Some(63),
            })
            .expect("system stats log should succeed");

        let contents =
            std::fs::read_to_string(logger.summary().run_path).expect("run log should be readable");
        assert!(contents.contains("system_stats_snapshot"));
        assert!(contents.contains("cpu_pressure=21%"));
        assert!(contents.contains("memory_pressure=63%"));
    }

    #[test]
    fn claude_native_log_writes_summary_counts() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut logger =
            RunLogger::start(temp_dir.path(), 2, LogVerbosity::Info).expect("logger should start");

        logger
            .log_claude_native_summary(&ClaudeNativeOverlaySummary {
                applied: 1,
                fallback_to_compatibility: 2,
                warnings: vec!["missing".to_string()],
            })
            .expect("native summary log should succeed");

        let contents =
            std::fs::read_to_string(logger.summary().run_path).expect("run log should be readable");
        assert!(contents.contains("claude_native_summary"));
        assert!(contents.contains("applied=1"));
        assert!(contents.contains("fallback_to_compatibility=2"));
    }

    #[test]
    fn codex_native_log_writes_summary_counts() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut logger =
            RunLogger::start(temp_dir.path(), 2, LogVerbosity::Info).expect("logger should start");

        logger
            .log_codex_native_summary(&CodexNativeOverlaySummary {
                applied: 2,
                fallback_to_compatibility: 1,
                warnings: vec!["missing".to_string()],
            })
            .expect("native summary log should succeed");

        let contents =
            std::fs::read_to_string(logger.summary().run_path).expect("run log should be readable");
        assert!(contents.contains("codex_native_summary"));
        assert!(contents.contains("applied=2"));
        assert!(contents.contains("fallback_to_compatibility=1"));
    }

    #[test]
    fn pi_native_log_writes_summary_counts() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut logger =
            RunLogger::start(temp_dir.path(), 2, LogVerbosity::Info).expect("logger should start");

        logger
            .log_pi_native_summary(&PiNativeOverlaySummary {
                applied: 3,
                fallback_to_compatibility: 1,
                warnings: vec!["missing".to_string()],
            })
            .expect("native summary log should succeed");

        let contents =
            std::fs::read_to_string(logger.summary().run_path).expect("run log should be readable");
        assert!(contents.contains("pi_native_summary"));
        assert!(contents.contains("applied=3"));
        assert!(contents.contains("fallback_to_compatibility=1"));
    }

    #[test]
    fn notification_logs_capture_decisions_and_backend_results() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut logger =
            RunLogger::start(temp_dir.path(), 2, LogVerbosity::Info).expect("logger should start");
        let request = NotificationRequest {
            pane_id: "alpha:claude".into(),
            pane_title: "claude-main".to_string(),
            kind: NotificationKind::Completion,
            title: "Agent ready: claude-main".to_string(),
            body: "The agent returned to an idle state.".to_string(),
            workspace_path: None,
        };

        logger
            .log_notification_decision(&NotificationDecision {
                pane_id: "alpha:claude".into(),
                kind: NotificationKind::Completion,
                reason: NotificationDecisionReason::WorkingBecameReady,
                request: Some(request.clone()),
            })
            .expect("decision log should succeed");
        logger
            .log_notification_backend_selected(
                &request,
                &NotificationDispatchReceipt {
                    backend_name: "fallback".to_string(),
                },
            )
            .expect("backend selected log should succeed");
        logger
            .log_notification_backend_failure(
                &request,
                &NotificationError::Unavailable("notify-send is not installed".to_string()),
            )
            .expect("backend failure log should succeed");

        let contents =
            std::fs::read_to_string(logger.summary().run_path).expect("run log should be readable");
        assert!(contents.contains("notification_decision"));
        assert!(contents.contains("action=emit"));
        assert!(contents.contains("reason=working_became_ready"));
        assert!(contents.contains("notification_backend_selected"));
        assert!(contents.contains("backend=fallback"));
        assert!(contents.contains("notification_backend_failure"));
    }

    #[test]
    fn operator_alert_log_captures_source_and_level() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut logger =
            RunLogger::start(temp_dir.path(), 2, LogVerbosity::Info).expect("logger should start");

        logger
            .log_operator_alert(&OperatorAlert::new(
                OperatorAlertSource::PullRequests,
                OperatorAlertLevel::Warn,
                "PR lookup unavailable: GitHub CLI is not installed",
            ))
            .expect("operator alert log should succeed");

        let contents =
            std::fs::read_to_string(logger.summary().run_path).expect("run log should be readable");
        assert!(contents.contains("operator_alert"));
        assert!(contents.contains("source=pull_requests"));
        assert!(contents.contains("level=warn"));
        assert!(contents.contains("GitHub CLI is not installed"));
    }

    #[test]
    fn pull_request_lookup_log_records_workspace_and_outcome() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut logger =
            RunLogger::start(temp_dir.path(), 2, LogVerbosity::Info).expect("logger should start");

        logger
            .log_pull_request_lookup(
                Path::new("/tmp/alpha"),
                &PullRequestLookup::Available(PullRequestData {
                    number: 42,
                    title: "Add runtime loop".to_string(),
                    url: "https://example.com/pr/42".to_string(),
                    repository: "foreman".to_string(),
                    branch: "feat/runtime".to_string(),
                    base_branch: "main".to_string(),
                    author: "alex".to_string(),
                    status: PullRequestStatus::Open,
                }),
            )
            .expect("pull request lookup log should succeed");

        let contents =
            std::fs::read_to_string(logger.summary().run_path).expect("run log should be readable");
        assert!(contents.contains("pull_request_lookup"));
        assert!(contents.contains("workspace=/tmp/alpha"));
        assert!(contents.contains("outcome=available:#42:OPEN"));
    }
}
