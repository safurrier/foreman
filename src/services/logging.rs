use crate::config::RuntimeConfig;
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
                "bootstrap_complete config={} poll_interval_ms={} capture_lines={} popup={} notifications_enabled={}",
                runtime.config_file.display(),
                runtime.poll_interval_ms,
                runtime.capture_lines,
                runtime.popup,
                runtime.notifications_enabled
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
    use crate::config::RuntimeConfig;
    use std::path::Path;
    use tempfile::tempdir;

    fn runtime_config(log_dir: &Path) -> RuntimeConfig {
        RuntimeConfig {
            config_file: Path::new("/tmp/config.toml").to_path_buf(),
            log_dir: log_dir.to_path_buf(),
            poll_interval_ms: 1_000,
            capture_lines: 200,
            popup: false,
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
    }
}
