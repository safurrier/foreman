use crate::app::InventorySummary;
use crate::config::RuntimeConfig;
use crate::integrations::ClaudeNativeOverlaySummary;
use crate::services::notifications::{
    NotificationDecision, NotificationDispatchReceipt, NotificationError, NotificationRequest,
};
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
    run_file: File,
    latest_file: File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunLogSummary {
    pub run_path: PathBuf,
    pub latest_path: PathBuf,
}

impl RunLogger {
    pub fn start(log_dir: &Path, retain_run_logs: usize) -> io::Result<Self> {
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

    pub fn log_bootstrap(&mut self, runtime: &RuntimeConfig) -> io::Result<()> {
        self.write_line(
            "INFO",
            &format!(
                "bootstrap_complete config={} poll_interval_ms={} capture_lines={} popup={} pr_monitoring_enabled={} pr_poll_interval_ms={} notifications_enabled={} tmux_socket={}",
                runtime.config_file.display(),
                runtime.poll_interval_ms,
                runtime.capture_lines,
                runtime.popup,
                runtime.pull_request_monitoring_enabled,
                runtime.pull_request_poll_interval_ms,
                runtime.notifications_enabled,
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

    pub fn log_tmux_error(&mut self, error: &str) -> io::Result<()> {
        self.write_line("WARN", &format!("tmux_bootstrap_error {error}"))
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
    use crate::app::{InventorySummary, NotificationKind};
    use crate::config::RuntimeConfig;
    use crate::integrations::ClaudeNativeOverlaySummary;
    use crate::services::notifications::{
        NotificationDecision, NotificationDecisionReason, NotificationDispatchReceipt,
        NotificationError, NotificationRequest,
    };
    use std::path::Path;
    use tempfile::tempdir;

    fn runtime_config(log_dir: &Path) -> RuntimeConfig {
        RuntimeConfig {
            config_file: Path::new("/tmp/config.toml").to_path_buf(),
            log_dir: log_dir.to_path_buf(),
            tmux_socket: None,
            claude_native_dir: None,
            poll_interval_ms: 1_000,
            capture_lines: 200,
            popup: false,
            pull_request_monitoring_enabled: true,
            pull_request_poll_interval_ms: 30_000,
            notifications_enabled: true,
            log_retention: 2,
        }
    }

    #[test]
    fn start_creates_run_log_and_latest_log() {
        let temp_dir = tempdir().expect("temp dir should exist");

        let mut logger = RunLogger::start(temp_dir.path(), 5).expect("logger should start");
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

        let _logger = RunLogger::start(temp_dir.path(), 2).expect("logger should start");

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
        let mut logger = RunLogger::start(temp_dir.path(), 2).expect("logger should start");

        logger
            .log_bootstrap(&runtime)
            .expect("bootstrap log should succeed");

        let contents =
            std::fs::read_to_string(logger.summary().run_path).expect("run log should be readable");
        assert!(contents.contains("bootstrap_complete"));
        assert!(contents.contains("poll_interval_ms=1000"));
        assert!(contents.contains("pr_monitoring_enabled=true"));
        assert!(contents.contains("pr_poll_interval_ms=30000"));
    }

    #[test]
    fn inventory_log_writes_visible_counts() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut logger = RunLogger::start(temp_dir.path(), 2).expect("logger should start");

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
    fn claude_native_log_writes_summary_counts() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut logger = RunLogger::start(temp_dir.path(), 2).expect("logger should start");

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
    fn notification_logs_capture_decisions_and_backend_results() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut logger = RunLogger::start(temp_dir.path(), 2).expect("logger should start");
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
}
